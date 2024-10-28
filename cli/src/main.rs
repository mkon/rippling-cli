mod commands;
pub mod persistence;

use std::{
    fs::{self, File},
    io::IsTerminal,
    sync::OnceLock,
};

use clap::Parser;
use commands::Commands;
use directories::ProjectDirs;

static INTERACTIVE: OnceLock<bool> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> commands::Result<()> {
    init_logging();
    let cli = Cli::parse();
    commands::execute(&cli.command)
}

fn is_interactive() -> bool {
    *INTERACTIVE.get_or_init(|| std::io::stdout().is_terminal())
}

fn init_logging() {
    if let Some(proj_dirs) = ProjectDirs::from("rs", "", "rippling-cli") {
        let dir = proj_dirs.config_dir();
        fs::create_dir_all(dir).unwrap();
        let file = File::create(dir.join("default.log")).unwrap();
        let pipe = env_logger::Target::Pipe(Box::new(file));
        env_logger::Builder::from_default_env().target(pipe).init();
    }
}
