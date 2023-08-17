use clap::{Parser, Subcommand};
use inquire::Text;
use rippling_api::mfa;

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    pub facility: Facility,
}

#[derive(Debug, Parser)]
pub struct Token {
    /// Optional MFA code - If omitted you will be prompted instead
    value: Option<String>,
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
        Facility::Token(token) => token_flow(token.value.clone()),
        Facility::Email => request_flow("EMAIL"),
        Facility::Mobile => request_flow("PHONE_TEXT"),
    }
}

fn request_flow(facility: &str) {
    let session = &super::get_session();
    super::wrap_in_spinner(|| mfa::request(session, facility), |r| r.message);
    let code = request_code();
    super::wrap_in_spinner(|| mfa::submit(session, facility, &code), |r| r.message);
}

fn token_flow(token: Option<String>) {
    let code: String;

    if let Some(t) = token {
        code = t;
    } else {
        code = request_code();
    }

    super::wrap_in_spinner(
        || mfa::token(&super::get_session(), &code),
        |r| if r { "Code valid".into() } else { "Code invalid".into() },
    )
}

fn request_code() -> String {
    Text::new("Enter the code").prompt().unwrap()
}
