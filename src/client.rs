mod authenticated;
mod public;
mod session;

use std::collections::HashMap;

pub use authenticated::Client as AuthenticatedClient;
pub use authenticated::TimeTrackEntry;
pub use public::Client as PublicClient;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError {
        status: u16,
        data: HashMap<String, serde_json::Value>,
    },
    MissingActivePolicy,
    Wrapping(Box<dyn std::error::Error>),
    UnexpectedPayload,
    UnhandledStatus(u16),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        dbg!(self);
        match self {
            Self::ApiError { status, data } => match data.get("detail") {
                Some(string) => write!(f, "{string}"),
                None => write!(f, "Unexpected response status {status}"),
            },
            Self::MissingActivePolicy => write!(f, "No active policy"),
            Self::UnexpectedPayload => write!(f, "Unexpected account info response"),
            Self::UnhandledStatus(status) => write!(f, "Unexpected response status {status}"),
            Self::Wrapping(err) => write!(f, "{err}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Wrapping(Box::new(value))
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::Wrapping(Box::new(value))
    }
}

impl From<reqwest::blocking::Response> for Error {
    fn from(res: reqwest::blocking::Response) -> Self {
        match res.headers().get("Content-Type") {
            Some(val) => {
                if val.to_str().unwrap().contains("application/json") {
                    Error::ApiError {
                        status: res.status().as_u16(),
                        data: res.json::<HashMap<String, serde_json::Value>>().unwrap(),
                    }
                } else {
                    Error::UnhandledStatus(res.status().as_u16())
                }
            }
            None => Error::UnhandledStatus(res.status().as_u16()),
        }
    }
}
