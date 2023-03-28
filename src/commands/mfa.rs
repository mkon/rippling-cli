use crate::client;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    pub facility: Facility,
}

#[derive(Debug, Subcommand)]
pub enum Facility {
    /// For use with a token generator like Google Authenticator
    Token,
    /// Enter the code which will be sent to your email address
    Email,
    /// Enter the code which will be sent to your Phone (SMS)
    Mobile,
}

pub fn execute(cmd: &Command) {
    match cmd.facility {
        Facility::Token => token_flow(),
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

fn token_flow() {
    let code = super::ask_user_input("Enter the code");
    super::wrap_in_spinner(
        || client::mfa::token(&super::get_session(), &code),
        |r| if r { "Code valid".into() } else { "Code invalid".into() },
    )
}
