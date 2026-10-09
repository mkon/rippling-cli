use clap::Parser;
use rippling_api::Client;
use std::result::Result as StdResult;
use time::SignedDuration;

use crate::{persistence, spinner_wrap};

/// Manually add entry for a day
#[derive(Debug, Parser)]
pub struct Command {
    #[arg(long)]
    pub max_break: u16,

    #[arg(long)]
    pub max_work: u16,

    #[arg(long)]
    pub warn: u16,

    #[arg(long)]
    pub enforce: bool,
}

pub(crate) fn execute(cmd: &Command) -> StdResult<(), super::Error> {
    if cmd.warn >= cmd.max_break || cmd.warn >= cmd.max_work {
        return Err(super::Error::WarnTooLarge);
    }
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
                    println!("On break too long!");
                    if cmd.enforce {
                        client.end_break(&entry.id, &br.break_type_id)?;
                        notify("Break ended automatically!");
                    }
                } else if elapsed >= max_break_duration - warn {
                    notify("End break soon!")
                }
            } else {
                let elapsed = entry.work_duration();
                if elapsed >= max_work_duration {
                    println!("Working too long!");
                    if cmd.enforce {
                        client.end_clock(&entry.id)?;
                        notify("Clocked out automatically!");
                    }
                } else if elapsed >= max_work_duration - warn {
                    notify("clock out soon!")
                }
            }
        }
        None => println!("Not clocked in!"),
    }
    Ok(())
}

fn notify(msg: &str) {
    let script = format!("display notification {msg:?} with title \"Rippling-Cli\"");
    let _ = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .status();
}
