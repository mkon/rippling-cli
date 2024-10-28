pub mod live;
pub mod manual_entry;
pub mod pto;

use clap::Subcommand;
use core::time::Duration;
use indicatif::ProgressBar;
use rippling_api::{self, Client};
use time::{macros::format_description, Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

use crate::persistence::State;

use self::pto::CheckOutcome;

const FORMAT_R: &[time::format_description::FormatItem] = format_description!("[hour]:[minute]");

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Configure this client
    Configure {
        #[command(subcommand)]
        command: ConfigureCommands,
    },

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

    /// Manually add entry for a day
    Manual(manual_entry::Command),
}

#[derive(Debug, Subcommand)]
pub enum ConfigureCommands {
    AccessToken { value: String },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError(rippling_api::Error),
    AlreadyOnBreak,
    NotClockedIn,
    NotOnBreak,
    NoManualBreakType,
    UnexpectedResponse,
    NoWorkingDay(CheckOutcome),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError(e) => write!(f, "{e}"),
            Self::AlreadyOnBreak => write!(f, "Already on a break"),
            Self::NotClockedIn => write!(f, "Not clocked in"),
            Self::NotOnBreak => write!(f, "Not on a break"),
            Self::NoManualBreakType => write!(f, "No manual break type"),
            Self::UnexpectedResponse => write!(f, "Unexpected response received"),
            Self::NoWorkingDay(r) => match r {
                CheckOutcome::Leave => write!(f, "You are on PTO"),
                CheckOutcome::Holiday(h) => write!(f, "It is a holiday ({})", h.name),
                CheckOutcome::Weekend(d) => write!(f, "It is a weekend ({})", d),
                _ => panic!("Unhandled enum match"),
            },
        }
    }
}

impl From<rippling_api::Error> for Error {
    fn from(value: rippling_api::Error) -> Self {
        Error::ApiError(value)
    }
}

pub fn execute(command: &Commands) -> Result<()> {
    match command {
        Commands::ClockIn => live::clock_in(),
        Commands::ClockOut => live::clock_out(),
        Commands::Status => live::status(),
        Commands::StartBreak => live::start_break(),
        Commands::EndBreak => live::end_break(),
        Commands::Configure { command } => match command {
            ConfigureCommands::AccessToken { value } => set_access_token(value),
        },
        Commands::Manual(cmd) => manual_entry::execute(cmd),
    }
}

#[macro_export]
macro_rules! spinner_wrap {
    ( $res: expr ) => {{
        if crate::is_interactive() {
            {
                let spinner = crate::commands::start_spinner();
                let result = $res;
                spinner.finish_and_clear();
                result
            }
        } else {
            $res
        }
    }};
}

fn set_access_token(token: &str) -> Result<()> {
    let client: Client = Client::new(token.to_string());
    let info = spinner_wrap!(client.account_info())?;
    let state = State {
        company_id: Some(info.role.company.id),
        role_id: Some(info.id),
        token: Some(token.to_string()),
    };
    state.store();
    Ok(())
}

fn today() -> Date {
    // This seems to crash sometimes ...
    // OffsetDateTime::now_local().unwrap().date()
    OffsetDateTime::now_utc().to_offset(local_offset()).date()
}

pub(crate) fn start_spinner() -> ProgressBar {
    let s = ProgressBar::new_spinner();
    s.set_message("Connecting with rippling...");
    s.enable_steady_tick(Duration::new(0, 100_000_000));
    s
}

fn format_hours(hours: f32) -> String {
    let h = hours.floor();
    let m = (hours.fract() * 60.0).floor();
    format!("{:1}:{:02}", h, m)
}

fn local_time_format(datetime: OffsetDateTime) -> String {
    datetime.to_offset(local_offset()).time().format(&FORMAT_R).unwrap()
}

fn local_offset() -> UtcOffset {
    local_offset_at(OffsetDateTime::now_utc())
}

fn local_offset_at(time: OffsetDateTime) -> UtcOffset {
    UtcOffset::local_offset_at(time).unwrap_or_else(|_| {
        let time_zone = tzdb::local_tz().unwrap().find_local_time_type(time.unix_timestamp());
        UtcOffset::from_whole_seconds(time_zone.unwrap().ut_offset()).unwrap()
    })
}

fn local_offset_estimated_at(time: PrimitiveDateTime) -> UtcOffset {
    let odt = time.assume_offset(local_offset());
    local_offset_at(odt)
}

#[cfg(test)]
mod tests {
    use time::{macros::datetime, UtcOffset};

    #[test]
    fn test_test_time_offset() {
        assert_eq!(
            super::local_offset_estimated_at(datetime!(2023-03-26 1:00)),
            UtcOffset::from_whole_seconds(3600).unwrap()
        );

        assert_eq!(
            super::local_offset_estimated_at(datetime!(2023-03-26 3:00)),
            UtcOffset::from_whole_seconds(7200).unwrap()
        );
    }
}
