mod commands;
mod persistence;

use std::fs::File;

use clap::Parser;
use commands::Commands;
use directories::ProjectDirs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    init_logging();
    let cli = Cli::parse();
    commands::execute(&cli.command)
}

fn init_logging() {
    if let Some(proj_dirs) = ProjectDirs::from("rs", "", "rippling-cli") {
        let dir = proj_dirs.config_dir();
        let file = File::create(dir.join("default.log")).unwrap();
        let pipe = env_logger::Target::Pipe(Box::new(file));
        env_logger::Builder::from_default_env().target(pipe).init();
    }
}
