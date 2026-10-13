use crate::spinner_wrap;
use crate::{cache, persistence};

use super::{format_duration, local_time_format};
use super::{Error, Result};
use rippling_api::Client;
use time::Duration;

pub fn status(ttl: Option<Duration>) -> Result<()> {
    let client: Client = persistence::state().into();
    let current = cache::current_time_entry(&client, ttl)?;
    match current {
        Some(entry) => {
            let mut msg = format!("Clocked in since {}", local_time_format(entry.start_time));

            // If on break, print the break start time
            if let Some(br) = entry.current_break() {
                msg.push_str(&format!(", started break at {}", local_time_format(br.start_time)));
            }

            // Print worked hours and breaks
            let worked_formatted = format_duration(entry.work_duration());
            let breaks_formatted = format_duration(entry.break_duration());
            msg.push_str(&format!(" (Worked: {worked_formatted}, Breaks: {breaks_formatted})"));

            println!("{msg}");
        }
        None => println!("Not clocked in!"),
    }
    Ok(())
}

pub fn status_compact(ttl: Option<Duration>) -> Result<()> {
    let client: Client = persistence::state().into();
    let current = cache::current_time_entry(&client, ttl)?;
    match current {
        Some(entry) => {
            // If on break, print the break start time
            if let Some(br) = entry.current_break() {
                println!("Break since {}", local_time_format(br.start_time));
            } else {
                let worked_formatted = format_duration(entry.work_duration());
                println!("Working for {worked_formatted}")
            }
        }
        None => println!("Clocked out"),
    }
    Ok(())
}

pub fn clock_in() -> Result<()> {
    let client: Client = persistence::state().into();
    let entry = spinner_wrap!(client.start_clock())?;
    cache::store(Some(&entry));
    println!("Clocked in since {}!", local_time_format(entry.start_time));
    Ok(())
}

pub fn clock_out() -> Result<()> {
    let client: Client = persistence::state().into();
    spinner_wrap!({
        let current = client.current_time_entry()?;
        match current {
            Some(entry) => {
                client.end_clock(&entry.id)?;
                cache::store(None);
                println!("Clocked out!");
                Ok(())
            }
            None => Err(Error::NotClockedIn),
        }
    })
}

pub fn start_break() -> Result<()> {
    let client: Client = persistence::state().into();
    spinner_wrap!({
        let current = client.current_time_entry()?;

        match current {
            None => Err(Error::NotClockedIn),
            Some(entry) => {
                if entry.current_break().is_some() {
                    Err(Error::AlreadyOnBreak)
                } else {
                    let break_policy = client.break_policy(&entry.active_policy.break_policy_id)?;
                    let break_type = break_policy.manual_break_type().ok_or(Error::NoManualBreakType)?;
                    let entry = client.start_break(&entry.id, &break_type.id)?;
                    cache::store(Some(&entry));
                    let brk = entry.current_break().unwrap().to_owned();
                    println!("Started break at {}!", local_time_format(brk.start_time));
                    Ok(())
                }
            }
        }
    })
}

pub fn end_break() -> Result<()> {
    let client: Client = persistence::state().into();
    spinner_wrap!({
        let current = client.current_time_entry()?;

        match current {
            None => Err(Error::NotClockedIn),
            Some(entry) => match entry.current_break() {
                None => Err(Error::NotOnBreak),
                Some(br) => {
                    let res = client.end_break(&entry.id, &br.break_type_id)?;
                    cache::store(Some(&res));
                    let brk = res.breaks.into_iter().last().ok_or(Error::UnexpectedResponse)?;
                    println!(
                        "Stopped break at {}, after {} hours!",
                        local_time_format(brk.end_time.unwrap()),
                        format_duration(brk.duration().unwrap())
                    );
                    Ok(())
                }
            },
        }
    })
}
