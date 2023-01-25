mod manual_entry;

use clap::Subcommand;

pub use manual_entry::{add_entry, InputShift};

use crate::client::{
    self,
    break_policy::{self},
    time_entries::{self, TimeEntryBreak},
    Session,
};

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Configure this client
    Configure {
        #[command(subcommand)]
        command: ConfigureCommands,
    },

    /// Authenticate against rippling
    Authenticate,

    /// Clock-in Status
    Status,

    /// Clock In
    #[clap(alias = "in")]
    ClockIn,

    /// Clock Out
    #[clap(alias = "out")]
    ClockOut,

    /// Start a break
    #[clap(alias = "sb", alias = "break")]
    StartBreak,

    /// Continue after a break
    #[clap(alias = "eb", alias = "continue")]
    EndBreak,

    /// Manually enter entry for today
    Today {
        /// Time ranges
        #[arg(value_parser = manual_entry::parse_input_shifts)]
        shifts: Vec<manual_entry::InputShift>,
    },

    /// Manually enter entry for today
    Yesterday {
        /// Time ranges
        #[arg(value_parser = manual_entry::parse_input_shifts)]
        shifts: Vec<manual_entry::InputShift>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigureCommands {
    Username { value: String },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError(client::Error),
    AlreadyOnBreak(TimeEntryBreak),
    NotClockedIn,
    NotOnBreak,
    NoManualBreakType,
    UnexpectedResponse,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError(e) => write!(f, "{e}"),
            Self::AlreadyOnBreak(_) => write!(f, "Already on a break"),
            Self::NotClockedIn => write!(f, "Not clocked in"),
            Self::NotOnBreak => write!(f, "Not on a break"),
            Self::NoManualBreakType => write!(f, "No manual break type"),
            Self::UnexpectedResponse => write!(f, "Unexpected response received"),
        }
    }
}

impl From<client::Error> for Error {
    fn from(value: client::Error) -> Self {
        Error::ApiError(value)
    }
}

pub fn start_break() -> Result<TimeEntryBreak> {
    let session = get_session();

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
    let session = get_session();

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

fn get_session() -> Session {
    #[cfg(not(test))]
    let session = Session::load();
    #[cfg(test)]
    let session = {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("company-id".into(), "role-id".into());
        session
    };
    session
}

#[cfg(test)]
mod tests {
    use mockito::mock;

    use super::*;

    fn mock_api(method: &str, path: &str) -> mockito::Mock {
        mock(method, path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_header("authorization", "Bearer access-token")
    }

    #[test]
    fn start_break_fails_when_not_authenticated() {
        let _m = mock_api("GET", "/time_tracking/api/time_entries?endTime=")
            .with_status(401)
            .with_body(r#"{"details":"Not authenitcated"}"#)
            .create();

        let result = start_break();
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::ApiError(e) => match e {
                client::Error::ApiError {
                    status,
                    description: _,
                    json: _,
                } => assert_eq!(status, 401),
                _ => assert!(false, "Wrong error"),
            },
            _ => assert!(false, "Wrong error"),
        };
    }

    #[test]
    fn start_break_fails_when_not_clocked_in() {
        let _m = mock_api("GET", "/time_tracking/api/time_entries?endTime=").with_body("[]").create();

        let result = start_break();
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::NotClockedIn => (),
            _ => assert!(false, "Wrong error"),
        };
    }
}
