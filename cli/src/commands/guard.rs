use clap::Parser;
use rippling_api::Client;
use std::result::Result as StdResult;
use time::SignedDuration;

use crate::{
    commands::{format_hours, local_time_format},
    persistence, spinner_wrap,
};

/// Manually add entry for a day
#[derive(Debug, Parser)]
pub struct Command {
    #[arg(long)]
    pub max_break: u16,

    #[arg(long)]
    pub max_work: u16,

    #[arg(long)]
    pub warn: u16,
}

pub(crate) fn execute(cmd: &Command) -> StdResult<(), super::Error> {
    let max_break_duration = SignedDuration::minutes(cmd.max_break.into());
    let max_work_duration = SignedDuration::minutes(cmd.max_work.into());
    let warn = SignedDuration::minutes(cmd.warn.into());
    let client: Client = persistence::state().into();
    let current = spinner_wrap!(client.current_time_entry())?;
    match current {
        Some(entry) => {
            // If on break, print the break start time
            if let Some(br) = entry.current_break() {
                let elapsed = br.duration().unwrap();
                if elapsed >= max_break_duration {
                    println!("On break too long!")
                } else if elapsed >= max_break_duration - warn {
                    println!("End break soon!")
                }
            } else {
                let elapsed = entry.work_duration().unwrap();
                if elapsed >= max_work_duration {
                    println!("Working too long!")
                } else if elapsed >= max_break_duration - warn {
                    println!("clock out soon!")
                }
            }
        }
        None => println!("Not clocked in!"),
    }
    Ok(())
}
