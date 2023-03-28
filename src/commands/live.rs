use super::{format_hours, local_time_format};
use super::{Error, Result};
use crate::client::{break_policy, time_entries};
use spinner_macro::spinner_wrap;
use std::time::Duration;

#[spinner_wrap]
pub fn status() -> Result<String> {
    Ok(match time_entries::current_entry(&super::get_session())? {
        Some(entry) => {
            let mut msg = format!("Clocked in since {}", local_time_format(entry.start_time));

            let regular_hours_in_seconds = (entry.regular_hours * 3600.0) as u64;
            let regular_hours_formatted = format_seconds_as_human_readable(regular_hours_in_seconds);

            let unpaid_break_hours_in_seconds = (entry.unpaid_break_hours * 3600.0) as u64;
            let unpaid_break_hours_formatted = format_seconds_as_human_readable(unpaid_break_hours_in_seconds);

            // If on break, print the break start time
            if let Some(br) = entry.current_break() {
                msg.push_str(&format!(", started break at {}", local_time_format(br.start_time)));
            }

            // Print regular hours and breaks
            msg.push_str(&format!(
                " (Regular hours: {}, Breaks: {})",
                regular_hours_formatted, unpaid_break_hours_formatted
            ));

            msg
        }
        None => "Not clocked in!".to_owned(),
    })
}

fn format_seconds_as_human_readable(seconds: u64) -> String {
    let breaks_duration = Duration::from_secs(seconds);
    format!(
        "{:02}h {:02}m",
        breaks_duration.as_secs() / 3600,
        (breaks_duration.as_secs() / 60) % 60
    )
}

#[spinner_wrap]
pub fn clock_in() -> Result<String> {
    let entry = time_entries::start_clock(&super::get_session())?;
    Ok(format!("Clocked in since {}!", local_time_format(entry.start_time)))
}

#[spinner_wrap]
pub fn clock_out() -> Result<String> {
    let session = super::get_session();
    match time_entries::current_entry(&session)? {
        Some(entry) => {
            time_entries::end_clock(&session, &entry.id)?;
            Ok(String::from("Clocked out!"))
        }
        None => Err(Error::NotClockedIn),
    }
}

#[spinner_wrap]
pub fn start_break() -> Result<String> {
    let session = super::get_session();

    match time_entries::current_entry(&session)? {
        None => Err(Error::NotClockedIn),
        Some(entry) => match entry.current_break() {
            Some(br) => Err(Error::AlreadyOnBreak(br.to_owned())),
            None => {
                let break_policy = break_policy::fetch(&session, &entry.active_policy.break_policy_id)?;
                let break_type = break_policy.manual_break_type().ok_or(Error::NoManualBreakType)?;
                let entry = time_entries::start_break(&session, &entry.id, &break_type.id)?;
                let brk = entry.current_break().unwrap().to_owned();
                Ok(format!("Started break at {}!", local_time_format(brk.start_time)))
            }
        },
    }
}

#[spinner_wrap]
pub fn end_break() -> Result<String> {
    let session = super::get_session();

    let current = time_entries::current_entry(&session)?;
    match current {
        None => Err(Error::NotClockedIn),
        Some(entry) => match entry.current_break() {
            None => Err(Error::NotOnBreak),
            Some(br) => {
                let res = time_entries::end_break(&session, &entry.id, &br.break_type_id)?;
                let brk = res.breaks.into_iter().last().ok_or(Error::UnexpectedResponse)?;
                Ok(format!(
                    "Stopped break at {}, after {} hours!",
                    local_time_format(brk.end_time.unwrap()),
                    format_hours(brk.duration().unwrap().whole_minutes() as f32 / 60.0)
                ))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use utilities::mocking;

    use super::*;

    #[test]
    fn start_break_fails_when_not_authenticated() {
        let _m = mocking::rippling("GET", "/time_tracking/api/time_entries?endTime=")
            .with_status(401)
            .with_body(r#"{"details":"Not authenitcated"}"#)
            .create();

        let result = start_break();
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::ApiError(e) => match e {
                crate::client::Error::ApiError { status, description: _, json: _ } => assert_eq!(status, 401),
                _ => assert!(false, "Wrong error"),
            },
            _ => assert!(false, "Wrong error"),
        };
    }

    #[test]
    fn start_break_fails_when_not_clocked_in() {
        let _m = mocking::rippling("GET", "/time_tracking/api/time_entries?endTime=")
            .with_body("[]")
            .create();

        let result = start_break();
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::NotClockedIn => (),
            _ => assert!(false, "Wrong error"),
        };
    }
}
