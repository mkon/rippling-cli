mod client;
mod persistance;

use std::io;

use clap::{Parser, Subcommand};
use client::{PublicClient, AuthenticatedClient, TimeTrackEntry};
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
}

#[derive(Debug, Subcommand)]
enum ConfigureCommands {
    Username { value: String },
}

fn main() {
    let cli = Cli::parse();
    let mut cfg = Settings::load();

    match &cli.command {
        Commands::Authenticate => {
            authenticate(&cfg);
        }
        Commands::Status => {
            tt_status(&cfg);
        }
        Commands::Test => {
            call_me(&cfg);
        }
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
            client.setup_company_any_role().expect("Failed to query account info");
            client.save();
        },
        _ => println!("Authentication failed")
    }
}

fn call_me(cfg: &Settings) {
    let client = client_from_config(cfg);

    client.account_info().unwrap();
    client.current_user();
}

fn tt_status(cfg: &Settings) {
    let client = client_from_config(cfg);

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let result: Vec<TimeTrackEntry> = client.tt_entries().expect("Failed to query time tracking");
    if result.is_empty() {
        sp.stop_with_message("Not clocked in!".into());
    } else {
        sp.stop_with_message(format!("Clocked in since {}!", result[0].start_time.format("%R")));
    }
}

fn client_from_config(_: &Settings) -> AuthenticatedClient {
    AuthenticatedClient::load()
}

fn ask_user_input(prompt: &str) -> String {
    println!("{prompt}");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_owned()
}
