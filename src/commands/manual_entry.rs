use chrono::{DateTime, Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveTime, Offset, TimeZone, Timelike};
use regex::Regex;
use std::result::Result as StdResult;

use super::Result;

use crate::client::{
    self,
    break_policy::{self, BreakPolicy},
    time_entries::{NewTimeEntry, TimeEntry},
};

#[derive(Clone, Debug)]
pub struct InputShift {
    start_time: NaiveTime,
    end_time: NaiveTime,
}

pub fn add_entry(date: NaiveDate, shifts: &Vec<InputShift>) -> Result<TimeEntry> {
    let session = super::get_session();
    let policy = break_policy::active_policy(&session)?;
    let break_policy = break_policy::fetch(&session, &policy.break_policy)?;

    let mut entry = NewTimeEntry::new();

    for shift in shifts {
        entry.add_shift(
            naive_to_fixed_datetime(date, shift.start_time),
            naive_to_fixed_datetime(date, shift.end_time),
        );
    }
    set_minimum_breaks(&mut entry, &break_policy);
    Ok(client::time_entries::create_entry(&session, &entry)?)
}

fn naive_to_fixed_datetime(date: NaiveDate, time: NaiveTime) -> DateTime<FixedOffset> {
    let east = Local::now().offset().fix().local_minus_utc();
    let timezone = FixedOffset::east_opt(east).unwrap();
    timezone
        .with_ymd_and_hms(date.year(), date.month(), date.day(), time.hour(), time.minute(), time.second())
        .unwrap()
}

/// Sets the regulatory required minimum break per shift according to German labor law
fn set_minimum_breaks(entry: &mut NewTimeEntry, break_policy: &BreakPolicy) {
    let mut breaks: Vec<(DateTime<FixedOffset>, DateTime<FixedOffset>)> = Vec::new();
    let btype = break_policy.manual_break_type().unwrap();
    for shift in entry.shifts.iter() {
        let duration = shift.end_time - shift.start_time;
        let break_duration = minimum_break_for(duration);
        if break_duration.num_minutes() >= 0 {
            let break_start = shift.start_time + duration / 2 - break_duration / 2;
            let break_end = break_start + break_duration;
            breaks.push((break_start, break_end))
        }
    }
    for (start_time, end_time) in breaks {
        entry.add_break(btype.id.to_owned(), start_time, end_time)
    }
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

pub fn parse_input_shifts(s: &str) -> StdResult<InputShift, String> {
    let re = Regex::new(r"^(?P<h1>\d{1,2})(?::(?P<m1>\d{2}))?-(?P<h2>\d{1,2})(?::(?P<m2>\d{2}))?$").unwrap();
    if let Some(m) = re.captures(s) {
        let parsed: [Option<u32>; 4] =
            [m.name("h1"), m.name("m1"), m.name("h2"), m.name("m2")].map(|m| m.map(|v| v.as_str().parse().unwrap()));
        let start_time: NaiveTime =
            NaiveTime::from_hms_opt(parsed[0].unwrap(), parsed[1].unwrap_or(0), 0).ok_or(format!("Invalid time format"))?;
        let end_time: NaiveTime =
            NaiveTime::from_hms_opt(parsed[2].unwrap(), parsed[3].unwrap_or(0), 0).ok_or(format!("Invalid time format"))?;
        let shift = InputShift { start_time, end_time };
        Ok(shift)
    } else {
        Err("Shifts must be a range, for example 8:30-17:15".into())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

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
