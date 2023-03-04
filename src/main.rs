mod client;
mod commands;
mod persistence;

use clap::Parser;
use commands::Commands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    commands::execute(&cli.command)
}
