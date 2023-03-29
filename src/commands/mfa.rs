use crate::client;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    pub facility: Facility,
}

#[derive(Debug, Parser)]
pub struct Token {
    #[structopt(long)]
    token: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Facility {
    /// For use with a token generator like Google Authenticator
    Token(Token),
    /// Enter the code which will be sent to your email address
    Email,
    /// Enter the code which will be sent to your Phone (SMS)
    Mobile,
}

pub fn execute(cmd: &Command) {
    match &cmd.facility {
        Facility::Token(token) => token_flow(token.token.as_deref()),
        Facility::Email => request_flow("EMAIL"),
        Facility::Mobile => request_flow("PHONE_TEXT"),
    }
}

fn request_flow(facility: &str) {
    let session = &super::get_session();
    super::wrap_in_spinner(|| client::mfa::request(session, facility), |r| r.message);
    let code = super::ask_user_input("Enter the code");
    super::wrap_in_spinner(|| client::mfa::submit(session, facility, &code), |r| r.message);
}

fn token_flow(token: Option<&str>) {
    let input_code: Option<String>;
    let code: &str;

    if let Some(t) = token {
        code = t;
    } else {
        input_code = Some(super::ask_user_input("Enter the code"));
        code = input_code.as_ref().unwrap();
    }

    super::wrap_in_spinner(
        || client::mfa::token(&super::get_session(), &code),
        |r| if r { "Code valid".into() } else { "Code invalid".into() },
    )
}
