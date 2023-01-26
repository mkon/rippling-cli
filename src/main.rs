mod client;
mod commands;
mod persistence;

use std::{io};

use clap::Parser;
use client::{account_info, PublicClient};
use commands::{Commands, ConfigureCommands};
use persistence::Settings;
use spinners::{Spinner, Spinners};
use time::{macros::format_description, Date, OffsetDateTime, UtcOffset};

const FORMAT_R: &[time::format_description::FormatItem] = format_description!("[hour]:[minute]");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    let mut cfg = Settings::load();

    match &cli.command {
        Commands::Authenticate => authenticate(&cfg),
        Commands::ClockIn => run_clock_in(),
        Commands::ClockOut => run_clock_out(),
        Commands::Status => run_status(),
        Commands::StartBreak => run_break_start(),
        Commands::EndBreak => run_break_end(),
        Commands::Today { shifts } => run_add_entry(today(), shifts),
        Commands::Yesterday { shifts } => run_add_entry(Date::previous_day(today()).unwrap(), shifts),
        Commands::Configure { command } => {
            match command {
                ConfigureCommands::Username { value } => cfg.username = Some(value.clone()),
            }

            cfg.store();
        }
    };
}

fn today() -> Date {
    OffsetDateTime::now_local().unwrap().date()
}

fn authenticate(cfg: &Settings) {
    let username = match &cfg.username {
        None => ask_user_input("Enter your user name"),
        Some(value) => value.clone(),
    };
    let password = ask_user_input("Enter your password");

    let client = PublicClient::initialize_from_remote().unwrap();
    match client.authenticate(&username, &password) {
        Ok(mut session) => {
            let info = account_info::fetch(&session).expect("Failed to query account info");
            session.set_company_and_role(info.role.company.id, info.id);
            session.save();
        }
        _ => println!("Authentication failed"),
    }
}

fn run_add_entry(date: Date, shifts: &Vec<commands::InputShift>) {
    wrap_in_spinner(
        || commands::add_entry(date, shifts),
        |entry| {
            format!(
                "Added entry from {} to {}",
                local_time_format(entry.start_time),
                local_time_format(entry.end_time.unwrap())
            )
        },
    )
}

fn run_break_start() {
    wrap_in_spinner(commands::start_break, |br| {
        format!("Started break at {}!", local_time_format(br.start_time))
    })
}

fn run_break_end() {
    wrap_in_spinner(commands::end_break, |br| {
        format!(
            "Stopped break at {}, after {} hours!",
            local_time_format(br.end_time.unwrap()),
            format_hours(br.duration().unwrap().whole_minutes() as f32 / 60.0)
        )
    })
}

fn run_clock_in() {
    wrap_in_spinner(commands::clock_in, |entry| {
        format!("Clocked in since {}!", local_time_format(entry.start_time))
    })
}

fn run_clock_out() {
    wrap_in_spinner(commands::clock_out, |_entry| String::from("Clocked out!"))
}

fn run_status() {
    wrap_in_spinner(commands::status, |entry| match entry {
        Some(entry) => format!("Clocked in since {}!", local_time_format(entry.start_time)),
        None => String::from("Not clocked in!"),
    })
}

fn wrap_in_spinner<T, C, O>(cmd: C, ok: O)
where
    C: FnOnce() -> commands::Result<T>,
    O: FnOnce(T) -> String,
{
    wrap_in_spinner_or(cmd, ok, |e| format!("Error: {e}"))
}

fn wrap_in_spinner_or<T, C, O, E>(cmd: C, ok: O, er: E)
where
    C: FnOnce() -> commands::Result<T>,
    O: FnOnce(T) -> String,
    E: FnOnce(commands::Error) -> String,
{
    let mut sp = Spinner::new(Spinners::Dots9, String::from("Connecting with rippling"));
    match cmd() {
        Ok(t) => sp.stop_with_message(ok(t)),
        Err(e) => sp.stop_with_message(er(e)),
    }
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
    UtcOffset::current_local_offset().unwrap_or_else(|_| {
        let time_zone = tzdb::local_tz()
            .unwrap()
            .find_local_time_type(OffsetDateTime::now_utc().unix_timestamp());
        UtcOffset::from_whole_seconds(time_zone.unwrap().ut_offset()).unwrap()
    })
}

fn ask_user_input(prompt: &str) -> String {
    println!("> {prompt}");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_owned()
}
