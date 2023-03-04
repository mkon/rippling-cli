use clap::Subcommand;

use crate::client::{self, mfa::MfaInfo};

use super::Result;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Request { auth_option: String },
    Submit { auth_option: String, code: String },
}

pub fn execute(cmd: &Commands) {
    crate::wrap_in_spinner(
        || match cmd {
            Commands::Request { auth_option } => request(&auth_option),
            Commands::Submit { auth_option, code } => submit(&auth_option, &code),
        },
        |res| res.message,
    )
}

fn request(auth_option: &str) -> Result<MfaInfo> {
    Ok(client::mfa::request(&super::get_session(), auth_option)?)
}

fn submit(auth_option: &str, code: &str) -> Result<MfaInfo> {
    Ok(client::mfa::submit(&super::get_session(), auth_option, code)?)
}
