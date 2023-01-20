mod client;
mod persistence;

use std::io;

use clap::{Parser, Subcommand};
use client::{account_info, break_policy, time_entries, PublicClient, Session};
use persistence::Settings;
use spinners::{Spinner, Spinners};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
}

#[derive(Debug, Subcommand)]
enum ConfigureCommands {
    Username { value: String },
}

fn main() {
    let cli = Cli::parse();
    let mut cfg = Settings::load();

    match &cli.command {
        Commands::Authenticate => authenticate(&cfg),
        Commands::ClockIn => tt_clock_in(),
        Commands::ClockOut => tt_clock_out(),
        Commands::Status => tt_status(),
        Commands::StartBreak => tt_break_start(),
        Commands::EndBreak => tt_break_end(),
        Commands::Configure { command } => {
            match command {
                ConfigureCommands::Username { value } => cfg.username = Some(value.clone()),
            }

            cfg.store();
        }
    }
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

fn tt_break_start() {
    let session = Session::load();
    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());

    let current = time_entries::current_entry(&session).expect("Unable to fetch current entry");
    match current {
        None => sp.stop_with_message("Not clocked in!".into()),
        Some(entry) => match entry.current_break() {
            Some(br) => sp.stop_with_message(format!("Already on a break since {}!", br.start_time.format("%R"))),
            None => {
                let break_policy =
                    break_policy::fetch(&session, &entry.active_policy.break_policy_id).expect("Unable to fetch break policy");
                let break_type = break_policy.manual_break_type().expect("No manual break type");
                let entry = time_entries::start_break(&session, &entry.id, &break_type.id).unwrap();
                let br = entry.current_break().unwrap();
                sp.stop_with_message(format!("Started break at {}!", br.start_time.format("%R")))
            }
        },
    }
}

fn tt_break_end() {
    let session = Session::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let current = time_entries::current_entry(&session).expect("Unable to fetch current entry");
    match current {
        None => sp.stop_with_message("Not clocked in!".into()),
        Some(entry) => match entry.current_break() {
            None => sp.stop_with_message(format!("Not on a break!")),
            Some(br) => {
                let res = time_entries::end_break(&session, &entry.id, &br.break_type_id).unwrap();
                let br = res.breaks.last().unwrap();
                sp.stop_with_message(format!(
                    "Stopped break at {}, after {} hours!",
                    br.end_time.unwrap().format("%R"),
                    format_hours(br.duration().unwrap().num_minutes() as f32 / 60.0)
                ));
            }
        },
    }
}

fn tt_clock_in() {
    let session = Session::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    match time_entries::start_clock(&session) {
        Ok(entry) => sp.stop_with_message(format!("Clocked in since {}!", entry.start_time.format("%R"))),
        Err(err) => sp.stop_with_message(format!("Error: {err}!")),
    }
}

fn tt_clock_out() {
    let session = Session::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let entry = time_entries::current_entry(&session).unwrap();
    match entry {
        Some(entry) => match time_entries::end_clock(&session, &entry.id) {
            Ok(_) => sp.stop_with_message("Clocked out!".into()),
            Err(err) => sp.stop_with_message(format!("Error: {err}!")),
        },
        None => sp.stop_with_message("Not clocked in!".into()),
    }
}

fn tt_status() {
    let session = Session::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let entry = time_entries::current_entry(&session).unwrap();
    match entry {
        None => sp.stop_with_message("Not clocked in!".into()),
        Some(entry) => match entry.current_break() {
            None => sp.stop_with_message(format!(
                "Clocked in since {}, for {} regular hours!",
                entry.start_time.format("%R"),
                format_hours(entry.regular_hours)
            )),
            Some(br) => sp.stop_with_message(format!("On break since {}!", br.start_time.format("%R"))),
        },
    }
}

fn format_hours(hours: f32) -> String {
    let h = hours.floor();
    let m = (hours.fract() * 60.0).floor();
    format!("{:1}:{:02}", h, m)
}

fn ask_user_input(prompt: &str) -> String {
    println!("> {prompt}");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_owned()
}
