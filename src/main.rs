mod client;
mod persistance;

use std::io;

use clap::{Parser, Subcommand};
use client::{AuthenticatedClient, PublicClient, TimeTrackEntry};
use persistance::Settings;
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

    /// Test
    Test,

    /// Clock-in Status
    Status,

    /// Clock In
    #[clap(alias = "in")]
    ClockIn,

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
        Commands::Status => tt_status(),
        Commands::Test => test(),
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
        Ok(mut client) => {
            client
                .setup_company_and_role()
                .expect("Failed to query account info");
            client.save();
        }
        _ => println!("Authentication failed"),
    }
}

fn test() {
    let client = AuthenticatedClient::load();

    let info = client.account_info().unwrap();
    dbg!(&info);
}

fn tt_break_start() {
    let client = AuthenticatedClient::load();
    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());

    let policy = client.tt_active_policy().expect("Unable to fetch policy");
    let break_policy = client
        .tt_break_policy(&policy.break_policy_id)
        .expect("Unable to fetch break policy");
    let entries = client.tt_entries().expect("Unable to fetch current entry");
    match entries.get(0) {
        None => sp.stop_with_message("Not clocked in!".into()),
        Some(entry) => match entry.current_break() {
            Some(br) => sp.stop_with_message(format!(
                "Already on a break since {}!",
                br.start_time.format("%R")
            )),
            None => {
                let break_type = break_policy
                    .manual_break_type()
                    .expect("No manual break type");
                let entry = client.tt_break_start(&entry.id, &break_type.id).unwrap();
                let br = entry.current_break().unwrap();
                sp.stop_with_message(format!("Started break at {}!", br.start_time.format("%R")))
            }
        },
    }
}

fn tt_break_end() {
    let client = AuthenticatedClient::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let result: Vec<TimeTrackEntry> = client.tt_entries().unwrap();
    match result.get(0) {
        None => sp.stop_with_message("Not clocked in!".into()),
        Some(entry) => match entry.current_break() {
            None => sp.stop_with_message(format!("Not on a break!")),
            Some(br) => {
                let res = client.tt_break_end(&entry.id, &br.break_type_id).unwrap();
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
    let client = AuthenticatedClient::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    match client.tt_clock_start() {
        Ok(entry) => sp.stop_with_message(format!(
            "Clocked in since {}!",
            entry.start_time.format("%R")
        )),
        Err(err) => sp.stop_with_message(format!("Error: {err}!")),
    }
}

fn tt_status() {
    let client = AuthenticatedClient::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let result: Vec<TimeTrackEntry> = client.tt_entries().unwrap();
    match result.get(0) {
        None => sp.stop_with_message("Not clocked in!".into()),
        Some(entry) => match entry.current_break() {
            None => sp.stop_with_message(format!(
                "Clocked in since {}, for {} regular hours!",
                entry.start_time.format("%R"),
                format_hours(entry.regular_hours)
            )),
            Some(br) => {
                sp.stop_with_message(format!("On break since {}!", br.start_time.format("%R")))
            }
        },
    }
}

fn format_hours(hours: f32) -> String {
    let h = hours.floor();
    let m = (hours.fract() * 60.0).floor();
    format!("{:1}:{:02}", h, m)
}

fn ask_user_input(prompt: &str) -> String {
    println!("{prompt}");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_owned()
}
