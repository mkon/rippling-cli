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

pub type Result<T> = std::result::Result<T, Error>;
