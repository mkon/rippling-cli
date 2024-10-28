use std::thread;

use rippling_api::{pto::Holiday, Client};
use time::Date;

use crate::persistence;

use super::Result;

#[derive(Debug)]
pub enum CheckOutcome {
    Leave,
    Holiday(Holiday),
    WorkingDay,
    Weekend(time::Weekday),
}

pub fn check(date: Date) -> Result<CheckOutcome> {
    let tw = thread::spawn(move || is_weekend(date));
    let th = thread::spawn(move || check_holiday(date));
    let tl = thread::spawn(move || is_leave_request(date));

    if let Some(weekend) = tw.join().unwrap() {
        Ok(CheckOutcome::Weekend(weekend))
    } else if let Some(holiday) = th.join().unwrap()? {
        Ok(CheckOutcome::Holiday(holiday))
    } else if tl.join().unwrap()? {
        Ok(CheckOutcome::Leave)
    } else {
        Ok(CheckOutcome::WorkingDay)
    }
}

fn check_holiday(date: Date) -> Result<Option<Holiday>> {
    let client: Client = persistence::state().into();
    let cal = client.holiday_calendar()?;
    match cal.into_iter().find(|hy| hy.year as i32 == date.year()) {
        Some(year) => Ok(year
            .holidays
            .into_iter()
            .find(|h| h.start_date <= date && h.end_date >= date)),
        None => Ok(None),
    }
}

fn is_leave_request(date: Date) -> Result<bool> {
    let client: Client = persistence::state().into();
    let lr = client.leave_requests()?;
    let found = lr.into_iter().any(|r| r.start_date <= date && r.end_date >= date);
    Ok(found)
}

fn is_weekend(date: Date) -> Option<time::Weekday> {
    let day = date.weekday();
    match day {
        time::Weekday::Monday => None,
        time::Weekday::Tuesday => None,
        time::Weekday::Wednesday => None,
        time::Weekday::Thursday => None,
        time::Weekday::Friday => None,
        _ => Some(day),
    }
}
