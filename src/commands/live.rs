use super::{Error, Result};
use crate::client::{
    break_policy,
    time_entries::{self, TimeEntry, TimeEntryBreak},
};

pub fn status() -> Result<Option<TimeEntry>> {
    let session = super::get_session();

    Ok(time_entries::current_entry(&session)?)
}

pub fn clock_in() -> Result<TimeEntry> {
    let session = super::get_session();

    Ok(time_entries::start_clock(&session)?)
}

pub fn clock_out() -> Result<TimeEntry> {
    let session = super::get_session();

    match time_entries::current_entry(&session)? {
        Some(entry) => Ok(time_entries::end_clock(&session, &entry.id)?),
        None => Err(Error::NotClockedIn),
    }
}

pub fn start_break() -> Result<TimeEntryBreak> {
    let session = super::get_session();

    let current = time_entries::current_entry(&session)?;
    match current {
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

pub fn end_break() -> Result<TimeEntryBreak> {
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
