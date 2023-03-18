use super::{format_hours, local_time_format, wrap_in_spinner};
use super::{Error, Result};
use crate::client::{
    break_policy,
    time_entries::{self, TimeEntryBreak},
};

pub fn status() {
    wrap_in_spinner(
        || {
            let session = super::get_session();
            time_entries::current_entry(&session)
        },
        |entry| match entry {
            Some(entry) => format!("Clocked in since {}!", local_time_format(entry.start_time)),
            None => "Not clocked in!".to_owned(),
        },
    )
}

pub fn clock_in() {
    wrap_in_spinner(
        || {
            let session = super::get_session();
            time_entries::start_clock(&session)
        },
        |entry| format!("Clocked in since {}!", local_time_format(entry.start_time)),
    )
}

pub fn clock_out() {
    wrap_in_spinner(
        || {
            let session = super::get_session();
            match time_entries::current_entry(&session)? {
                Some(entry) => Ok(time_entries::end_clock(&session, &entry.id)?),
                None => Err(Error::NotClockedIn),
            }
        },
        |_entry| String::from("Clocked out!"),
    )
}

pub fn start_break() {
    wrap_in_spinner(do_start_break, |br| {
        format!("Started break at {}!", local_time_format(br.start_time))
    })
}

fn do_start_break() -> Result<TimeEntryBreak> {
    let session = super::get_session();

    match time_entries::current_entry(&session)? {
        None => Err(Error::NotClockedIn),
        Some(entry) => match entry.current_break() {
            Some(br) => Err(Error::AlreadyOnBreak(br.to_owned())),
            None => {
                let break_policy = break_policy::fetch(&session, &entry.active_policy.break_policy_id)?;
                let break_type = break_policy.manual_break_type().ok_or(Error::NoManualBreakType)?;
                let entry = time_entries::start_break(&session, &entry.id, &break_type.id)?;
                Ok(entry.current_break().unwrap().to_owned())
            }
        },
    }
}

pub fn end_break() {
    wrap_in_spinner(do_end_break, |br| {
        format!(
            "Stopped break at {}, after {} hours!",
            local_time_format(br.end_time.unwrap()),
            format_hours(br.duration().unwrap().whole_minutes() as f32 / 60.0)
        )
    })
}

fn do_end_break() -> Result<TimeEntryBreak> {
    let session = super::get_session();

    let current = time_entries::current_entry(&session)?;
    match current {
        None => Err(Error::NotClockedIn),
        Some(entry) => match entry.current_break() {
            None => Err(Error::NotOnBreak),
            Some(br) => {
                let res = time_entries::end_break(&session, &entry.id, &br.break_type_id)?;
                Ok(res.breaks.into_iter().last().ok_or(Error::UnexpectedResponse)?)
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

        let result = do_start_break();
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

        let result = do_start_break();
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::NotClockedIn => (),
            _ => assert!(false, "Wrong error"),
        };
    }
}
