use clap::Parser;
use rippling_api::{time_entries::TimeEntry, Client};
use std::result::Result as StdResult;
use time::SignedDuration;

use crate::{cache, persistence};

/// Warn before break and work limits are reached, and optionally enforce them
#[derive(Debug, Parser)]
pub struct Command {
    /// Maximum length of a break in minutes
    #[arg(long)]
    pub max_break: u16,

    /// Maximum worked time in minutes, excluding breaks
    #[arg(long)]
    pub max_work: u16,

    /// Minutes before a limit is reached to start warning
    #[arg(long)]
    pub warn: u16,

    /// End the break or clock out when a limit is exceeded
    #[arg(long)]
    pub enforce: bool,

    /// Max age of the cached time entry in seconds. Always refetched before warning or enforcing.
    #[arg(long, default_value_t = 300)]
    pub ttl: u32,
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Ok,
    BreakEndingSoon,
    BreakTooLong,
    WorkEndingSoon,
    WorkTooLong,
}

impl Command {
    fn check(&self, entry: &TimeEntry) -> Verdict {
        let (elapsed, max, soon, over) = match entry.current_break() {
            Some(br) => (
                br.duration().unwrap(),
                self.max_break,
                Verdict::BreakEndingSoon,
                Verdict::BreakTooLong,
            ),
            None => (
                entry.work_duration(),
                self.max_work,
                Verdict::WorkEndingSoon,
                Verdict::WorkTooLong,
            ),
        };
        let max = SignedDuration::minutes(max.into());
        if elapsed >= max {
            over
        } else if elapsed >= max - SignedDuration::minutes(self.warn.into()) {
            soon
        } else {
            Verdict::Ok
        }
    }
}

pub(crate) fn execute(cmd: &Command) -> StdResult<(), super::Error> {
    if cmd.warn >= cmd.max_break || cmd.warn >= cmd.max_work {
        return Err(super::Error::WarnTooLarge);
    }
    let client: Client = persistence::state().into();
    let mut current = cache::current_time_entry(&client, Some(SignedDuration::seconds(cmd.ttl.into())))?;
    // Never warn or enforce based on stale data
    if current.as_ref().is_some_and(|entry| cmd.check(entry) != Verdict::Ok) {
        current = cache::current_time_entry(&client, None)?;
    }
    let Some(entry) = current else { return Ok(()) };
    match cmd.check(&entry) {
        Verdict::Ok => {}
        Verdict::BreakEndingSoon => notify("End break soon!"),
        Verdict::BreakTooLong => {
            println!("On break too long!");
            if cmd.enforce {
                let br = entry.current_break().unwrap();
                let updated = client.end_break(&entry.id, &br.break_type_id)?;
                cache::store(Some(&updated));
                notify("Break ended automatically!");
            }
        }
        Verdict::WorkEndingSoon => notify("clock out soon!"),
        Verdict::WorkTooLong => {
            println!("Working too long!");
            if cmd.enforce {
                client.end_clock(&entry.id)?;
                cache::store(None);
                notify("Clocked out automatically!");
            }
        }
    }
    Ok(())
}

fn notify(msg: &str) {
    let script = format!("display notification {msg:?} with title \"Rippling-Cli\"");
    let _ = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .status();
}
