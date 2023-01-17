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
                .setup_company_any_role()
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

fn tt_clock_in() {
    let client = AuthenticatedClient::load();

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    match client.tt_clock_start() {
        Ok(entry) => sp.stop_with_message(format!(
            "Clocked in since {}!",
            entry.start_time.format("%R")
        )),
        Err(err) => sp.stop_with_message(format!("Error: {:?}!", err)),
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
                "Clocked in since {}!",
                entry.start_time.format("%R")
            )),
            Some(br) => {
                sp.stop_with_message(format!("On break since {}!", br.start_time.format("%R")))
            }
        },
    }
}

fn ask_user_input(prompt: &str) -> String {
    println!("{prompt}");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_owned()
}
