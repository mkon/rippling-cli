use clap::Subcommand;

use crate::client;

use super::Result;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Request { auth_option: String },
    Submit { auth_option: String, code: String },
    Token { code: String },
}

pub fn execute(cmd: &Commands) {
    super::wrap_in_spinner(
        || match cmd {
            Commands::Request { auth_option } => request(&auth_option),
            Commands::Submit { auth_option, code } => submit(&auth_option, &code),
            Commands::Token { code } => token(&code),
        },
        |msg| msg,
    )
}

fn request(auth_option: &str) -> Result<String> {
    Ok(client::mfa::request(&super::get_session(), auth_option)?.message)
}

fn submit(auth_option: &str, code: &str) -> Result<String> {
    Ok(client::mfa::submit(&super::get_session(), auth_option, code)?.message)
}

fn token(code: &str) -> Result<String> {
    let res = client::mfa::token(&super::get_session(), code)?;
    match res {
        true => Ok("Code valid".into()),
        false => Ok("Code invalid".into()),
    }
}
