mod client;
mod commands;
mod persistence;

use std::fs::File;

use clap::Parser;
use commands::Commands;
use directories::ProjectDirs;
use log::LevelFilter;
use simplelog::{WriteLogger, Config};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    if let Some(proj_dirs) = ProjectDirs::from("rs", "",  "rippling-cli") {
        let dir = proj_dirs.config_dir();
        let _ = WriteLogger::init(LevelFilter::Info, Config::default(), File::create(dir.join("default.log")).unwrap());
    }

    let cli = Cli::parse();

    commands::execute(&cli.command)
}
