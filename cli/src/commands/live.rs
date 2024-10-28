use crate::persistence;
use crate::spinner_wrap;

use super::{format_hours, local_time_format};
use super::{Error, Result};
use rippling_api::Client;

pub fn status() -> Result<()> {
    let client: Client = persistence::state().into();
    let current = spinner_wrap!(client.current_time_entry())?;
    match current {
        Some(entry) => {
            let mut msg = format!("Clocked in since {}", local_time_format(entry.start_time));

            // If on break, print the break start time
            if let Some(br) = entry.current_break() {
                msg.push_str(&format!(", started break at {}", local_time_format(br.start_time)));
            }

            // Print regular hours and breaks
            let regular_hours_formatted = format_hours(entry.regular_hours);
            let unpaid_break_hours_formatted = format_hours(entry.unpaid_break_hours);
            msg.push_str(&format!(
                " (Regular hours: {}, Breaks: {})",
                regular_hours_formatted, unpaid_break_hours_formatted
            ));

            println!("{msg}");
        }
        None => println!("Not clocked in!"),
    }
    Ok(())
}

pub fn clock_in() -> Result<()> {
    let client: Client = persistence::state().into();
    let entry = spinner_wrap!(client.start_clock())?;
    println!("Clocked in since {}!", local_time_format(entry.start_time));
    Ok(())
}

pub fn clock_out() -> Result<()> {
    let client: Client = persistence::state().into();
    let current = spinner_wrap!(client.current_time_entry())?;
    match current {
        Some(entry) => {
            client.end_clock(&entry.id)?;
            println!("Clocked out!");
            Ok(())
        }
        None => Err(Error::NotClockedIn),
    }
}

pub fn start_break() -> Result<()> {
    let client: Client = persistence::state().into();
    let current = spinner_wrap!(client.current_time_entry())?;

    match current {
        None => Err(Error::NotClockedIn),
        Some(entry) => match entry.current_break() {
            Some(_) => Err(Error::AlreadyOnBreak),
            None => {
                let break_policy = client.break_policy(&entry.active_policy.break_policy_id)?;
                let break_type = break_policy.manual_break_type().ok_or(Error::NoManualBreakType)?;
                let entry = client.start_break(&entry.id, &break_type.id)?;
                let brk = entry.current_break().unwrap().to_owned();
                println!("Started break at {}!", local_time_format(brk.start_time));
                Ok(())
            }
        },
    }
}

pub fn end_break() -> Result<()> {
    let client: Client = persistence::state().into();
    let current = spinner_wrap!(client.current_time_entry())?;

    match current {
        None => Err(Error::NotClockedIn),
        Some(entry) => match entry.current_break() {
            None => Err(Error::NotOnBreak),
            Some(br) => {
                let res = client.end_break(&entry.id, &br.break_type_id)?;
                let brk = res.breaks.into_iter().last().ok_or(Error::UnexpectedResponse)?;
                println!(
                    "Stopped break at {}, after {} hours!",
                    local_time_format(brk.end_time.unwrap()),
                    format_hours(brk.duration().unwrap().whole_minutes() as f32 / 60.0)
                );
                Ok(())
            }
        },
    }
}
