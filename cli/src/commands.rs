pub mod live;
pub mod manual_entry;
pub mod mfa;
pub mod pto;

use clap::Subcommand;
use core::time::Duration;
use indicatif::ProgressBar;
use inquire::{Password, Text};
use rippling_api::{self, Session};
use time::{macros::format_description, Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

use crate::persistence::Settings;

use self::pto::CheckOutcome;

const FORMAT_R: &[time::format_description::FormatItem] = format_description!("[hour]:[minute]");

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

    /// Manually add entry for a day
    Manual(manual_entry::Command),

    /// Multi Factor Authentication flows
    Mfa(mfa::Command),
}

#[derive(Debug, Subcommand)]
pub enum ConfigureCommands {
    Username { value: String },
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

pub fn execute(command: &Commands) {
    let mut cfg = Settings::load();

    match command {
        Commands::Authenticate => authenticate(&cfg),
        Commands::ClockIn => live::clock_in_spinner(),
        Commands::ClockOut => live::clock_out_spinner(),
        Commands::Status => live::status_spinner(),
        Commands::StartBreak => live::start_break_spinner(),
        Commands::EndBreak => live::end_break_spinner(),
        Commands::Configure { command } => {
            match command {
                ConfigureCommands::Username { value } => cfg.username = Some(value.clone()),
            }

            cfg.store();
        }
        Commands::Manual(cmd) => manual_entry::execute(cmd),
        Commands::Mfa(command) => mfa::execute(command),
    };
}

fn authenticate(cfg: &Settings) {
    let client = rippling_api::PublicClient::initialize_from_remote().unwrap();
    let username = match &cfg.username {
        None => Text::new("Enter your user name").prompt().unwrap(),
        Some(value) => value.clone(),
    };
    let password = Password::new("Enter your password")
        .without_confirmation()
        .prompt()
        .unwrap();

    let s = start_spinner();
    match client.authenticate(&username, &password) {
        Ok(mut session) => {
            let info = rippling_api::account_info::fetch(&session).expect("Failed to query account info");
            session.set_company_and_role(info.role.company.id, info.id);
            save_session(&session);
            s.finish_with_message("Authentication successfull");
        }
        _ => {
            s.finish_with_message("Authentication failed!");
        }
    }
}

fn get_session() -> Session {
    let session = {
        let state = crate::persistence::State::load();
        let mut s = Session::new(None, state.access_token.unwrap());
        s.company = state.company_id;
        s.role = state.role_id;
        s
    };
    session
}

fn save_session(session: &Session) {
    let state = crate::persistence::State {
        access_token: Some(session.access_token.clone()),
        company_id: session.company.clone(),
        role_id: session.role.clone(),
    };
    state.store();
}

fn today() -> Date {
    // This seems to crash sometimes ...
    // OffsetDateTime::now_local().unwrap().date()
    OffsetDateTime::now_utc().to_offset(local_offset()).date()
}

fn wrap_in_spinner<T, E, Fn, Ok>(f: Fn, ok: Ok)
where
    Fn: FnOnce() -> std::result::Result<T, E>,
    Ok: FnOnce(T) -> String,
    E: std::fmt::Display,
{
    wrap_in_spinner_or(f, ok, |e| format!("Error: {e}"))
}

fn wrap_in_spinner_or<T, E, Fn, Ok, Er>(f: Fn, ok: Ok, er: Er)
where
    Fn: FnOnce() -> std::result::Result<T, E>,
    Ok: FnOnce(T) -> String,
    Er: FnOnce(E) -> String,
{
    if super::INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        let s = start_spinner();
        match f() {
            Ok(t) => s.finish_with_message(ok(t)),
            Err(e) => s.finish_with_message(er(e)),
        }
    } else {
        match f() {
            Ok(t) => println!("{}", ok(t)),
            Err(e) => println!("{}", er(e)),
        }
    }
}

fn start_spinner() -> ProgressBar {
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
