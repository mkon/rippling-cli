pub mod account_info;
pub mod break_policy;
pub mod client;
mod error;
pub mod pto;
pub mod time_entries;

pub use client::Client;
pub use error::Error;

const DEFAULT_HOST: &str = "https://app.rippling.com";
const API_ROOT: &str = "/api/";

pub type Result<T> = std::result::Result<T, Error>;

fn default_host() -> url::Url {
    url::Url::parse(DEFAULT_HOST).unwrap()
}

fn default_root() -> url::Url {
    default_host().join(API_ROOT).unwrap()
}
