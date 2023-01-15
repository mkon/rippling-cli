mod client;
mod config;

use std::io;

use clap::{Parser, Subcommand};
use client::Client;
use config::MyConfig;
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
    ClientId { value: String },
    ClientSecret { value: String },
    Username { value: String },
}

fn main() {
    let cli = Cli::parse();
    let mut cfg = MyConfig::load();

    match &cli.command {
        Commands::Authenticate => {
            authenticate(&mut cfg);
        }
        Commands::Status => {
            tt_status(&mut cfg);
        }
        Commands::Test => {
            call_me(&mut cfg);
        }
        Commands::Configure { command } => {
            match command {
                ConfigureCommands::ClientId { value } => cfg.client_id = Some(value.clone()),
                ConfigureCommands::ClientSecret { value } => {
                    cfg.client_secret = Some(value.clone())
                }
                ConfigureCommands::Username { value } => cfg.username = Some(value.clone()),
            }

            cfg.store();
        }
    }
}

fn authenticate(cfg: &mut MyConfig) {
    let username = match &cfg.username {
        None => ask_user_input("Enter your user name"),
        Some(value) => value.clone(),
    };
    let password = ask_user_input("Enter your password");

    let mut client = client_from_config(cfg);
    client.authenticate(&username, &password);

    cfg.access_token = client.access_token.to_owned();
    cfg.refresh_token = client.refresh_token.to_owned();

    let info = client.account_info();
    cfg.company = Some(info[0].role.company.id.clone());
    cfg.employee = Some(info[0].id.clone());
    cfg.store();
}

fn call_me(cfg: &MyConfig) {
    let client = client_from_config(cfg);

    client.account_info();
    client.current_user();
    client.tt_entries();
}

fn tt_status(cfg: &MyConfig) {
    let client = client_from_config(cfg);

    let mut sp = Spinner::new(Spinners::Dots9, "Connecting with rippling".into());
    let entries = client.tt_entries();
    if entries.is_empty() {
        sp.stop_with_message("Not clocked in!".into());
    } else {
        sp.stop_with_message("Clocked in!".into());
    }
}

fn client_from_config(cfg: &MyConfig) -> Client {
    let mut client = Client::new(
        cfg.client_id.as_ref().unwrap(),
        cfg.client_secret.as_ref().unwrap(),
    );
    client.access_token = cfg.access_token.clone();
    client.refresh_token = cfg.refresh_token.clone();
    client.company = cfg.company.clone();
    client.role = cfg.employee.clone();
    client
}

fn ask_user_input(prompt: &str) -> String {
    println!("{prompt}");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_owned()
}
