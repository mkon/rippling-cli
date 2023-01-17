mod authenticated;
mod public;

pub use authenticated::Client as AuthenticatedClient;
pub use authenticated::TimeTrackEntry;
pub use public::Client as PublicClient;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        println!("reqwest error:");
        dbg!(&value);
        Error {}
    }
}
