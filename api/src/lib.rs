pub mod account_info;
pub mod break_policy;
mod error;
pub mod mfa;
pub mod pto;
mod public;
mod session;
pub mod time_entries;

pub use error::Error;
pub use public::Client as PublicClient;
pub use session::Session;

const DEFAULT_HOST: &str = "https://app.rippling.com";
const API_ROOT: &str = "/api/";

pub type Result<T> = std::result::Result<T, Error>;

fn default_host() -> url::Url {
    url::Url::parse(DEFAULT_HOST).unwrap()
}

fn default_root() -> url::Url {
    default_host().join(API_ROOT).unwrap()
}
