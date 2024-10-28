use clap::{arg, Parser};
use inquire::Confirm;
use regex::Regex;
use std::{result::Result as StdResult, thread};
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};

use crate::{persistence, spinner_wrap};

use super::pto::{self, CheckOutcome};
use super::Result;

use rippling_api::{self, break_policy::BreakPolicy, time_entries::NewTimeEntry, Client};

#[derive(Clone, Debug)]
pub struct TimeRange {
    start_time: Time,
    end_time: Time,
}

/// Manually add entry for a day
#[derive(Debug, Parser)]
pub struct Command {
    /// Defaults to 0 (today)
    #[arg(short, long)]
    pub days_ago: Option<u8>,
    /// Before submitting check for overlap with holidays, weekends or PTO
    #[arg(short, long)]
    pub check: bool,
    /// Bypass prompt with a yes answer
    #[arg(short, long)]
    pub yes: bool,
    #[arg(value_parser = parse_input_shifts)]
    pub ranges: Vec<TimeRange>,
}

/// Entrypoint for this module
pub fn execute(cmd: &Command) -> Result<()> {
    let date = super::today()
        .checked_sub(Duration::days(cmd.days_ago.unwrap_or(0) as i64))
        .unwrap();
    draft_entry(date, &cmd.ranges, cmd.check, cmd.yes)
}

fn draft_entry(date: Date, ranges: &Vec<TimeRange>, check: bool, yes: bool) -> Result<()> {
    let policy_thread = thread::spawn(|| -> StdResult<BreakPolicy, rippling_api::Error> {
        let client: Client = persistence::state().into();
        let policy = client.active_break_policy()?;
        client.break_policy(&policy.break_policy)
    });

    if check {
        let pto = pto::check(date).unwrap();
        if let CheckOutcome::WorkingDay = pto {
        } else {
            return Err(super::Error::NoWorkingDay(pto));
        }
    }

    // List of times where either work started or stopped
    let mut events: Vec<Time> = Vec::new();
    for range in ranges {
        events.push(range.start_time);
        events.push(range.end_time);
    }
    events = setup_minimum_breaks(&events);
    let mut events: Vec<OffsetDateTime> = events
        .into_iter()
        .map(|time| naive_to_fixed_datetime(date, time))
        .collect();

    let mut entry = NewTimeEntry::new();

    let start_time = events.remove(0);
    let end_time = events.pop().unwrap();
    entry.add_shift(start_time, end_time);

    let break_policy = policy_thread.join().unwrap().unwrap();
    let btype = break_policy.manual_break_type().unwrap();
    for pair in events.chunks(2) {
        entry.add_break(btype.id.to_owned(), pair[0], pair[1]);
    }

    if yes {
        submit_entry(entry)?;
    } else {
        if Confirm::new(&format!("Create entry {}?", entry)).prompt().unwrap() {
            submit_entry(entry)?;
        }
    }
    Ok(())
}

fn submit_entry(entry: NewTimeEntry) -> super::Result<()> {
    let client: Client = persistence::state().into();
    let entry = spinner_wrap!(client.create_time_entry(&entry))?;
    println!(
        "Added entry from {} to {}",
        super::local_time_format(entry.start_time),
        super::local_time_format(entry.end_time.unwrap())
    );
    Ok(())
}

fn naive_to_fixed_datetime(date: Date, time: Time) -> OffsetDateTime {
    let datetime: PrimitiveDateTime = PrimitiveDateTime::new(date, time);
    datetime.assume_offset(super::local_offset_estimated_at(datetime))
}

/// Sets the regulatory required minimum break per shift according to German labor law
fn setup_minimum_breaks(input: &Vec<Time>) -> Vec<Time> {
    assert!(input.len() % 2 == 0);
    let mut out: Vec<Time> = Vec::new();
    for pair in input.chunks_exact(2) {
        let duration = pair[1] - pair[0];
        let break_duration = minimum_break_for(duration);
        if break_duration.whole_minutes() > 0 {
            let break_start = pair[0] + duration / 2 - break_duration / 2;
            let break_end = break_start + break_duration;
            out.append(&mut vec![pair[0], break_start, break_end, pair[1]]);
        } else {
            out.append(&mut pair.to_vec());
        }
    }
    out
}

/// Calculate minimum break duration according to German labor law
fn minimum_break_for(duration: Duration) -> Duration {
    let mut dur = Duration::minutes(0);
    if duration > Duration::hours(6) {
        // after 6hrs up to 30min
        dur = (duration - Duration::hours(6))
            .min(Duration::minutes(30))
            .max(Duration::minutes(15));
    }
    if duration > Duration::hours(9) {
        // after 9hrs up to 30min + 15min
        dur = dur + (duration - Duration::hours(9)).min(Duration::minutes(15));
    }
    dur
}

pub fn parse_input_shifts(s: &str) -> StdResult<TimeRange, String> {
    let re = Regex::new(r"^(?P<h1>\d{1,2})(?::(?P<m1>\d{2}))?-(?P<h2>\d{1,2})(?::(?P<m2>\d{2}))?$").unwrap();
    if let Some(m) = re.captures(s) {
        let parsed: [Option<u8>; 4] =
            [m.name("h1"), m.name("m1"), m.name("h2"), m.name("m2")].map(|m| m.map(|v| v.as_str().parse().unwrap()));
        let start_time: Time = Time::from_hms(parsed[0].unwrap(), parsed[1].unwrap_or(0), 0).unwrap();
        let end_time: Time = Time::from_hms(parsed[2].unwrap(), parsed[3].unwrap_or(0), 0).unwrap();
        let shift = TimeRange { start_time, end_time };
        Ok(shift)
    } else {
        Err("Shifts must be a range, for example 8:30-17:15".into())
    }
}

#[cfg(test)]
mod tests {
    use time::Duration;

    #[test]
    fn minimum_break_for() {
        let examples = [
            (360, 0),  // 6h
            (365, 15), // 6h 5m
            (375, 15), // 6h 15m
            (420, 30), // 8h
            (540, 30), // 9h
            (545, 35), // 9h 5m
            (555, 45), // 9h 15m
            (600, 45), // 10h
        ];
        for (w, b) in examples.into_iter() {
            assert_eq!(super::minimum_break_for(Duration::minutes(w)), Duration::minutes(b));
        }
    }
}
