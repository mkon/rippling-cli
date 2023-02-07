pub mod account_info;
pub mod break_policy;
mod public;
mod session;
pub mod time_entries;

pub use public::Client as PublicClient;
use reqwest::blocking::RequestBuilder;
use reqwest::blocking::Response;
pub use session::Session;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError {
        status: u16,
        description: Option<String>,
        json: Option<serde_json::Value>,
    },
    Wrapping(Box<dyn std::error::Error>),
    UnexpectedPayload,
    UnhandledStatus(u16),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // dbg!(self);
        match self {
            Self::ApiError {
                status,
                description,
                json: _,
            } => match description {
                Some(string) => write!(f, "{string}"),
                None => write!(f, "Unexpected response status {status}"),
            },
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

impl From<reqwest::blocking::Response> for Error {
    fn from(res: reqwest::blocking::Response) -> Self {
        match res.headers().get("Content-Type") {
            Some(val) => {
                if val.to_str().unwrap().contains("application/json") {
                    let status = res.status().as_u16();
                    let data = res.json::<serde_json::Value>().unwrap();
                    match &data {
                        serde_json::Value::Array(list) if list.first().unwrap().is_string() => Error::ApiError {
                            status: status,
                            description: list.first().map(|v| v.as_str().unwrap().to_owned()),
                            json: Some(data),
                        },
                        serde_json::Value::Object(obj) if obj.contains_key("detail") => Error::ApiError {
                            status: status,
                            description: obj["detail"].as_str().map(|v| v.to_owned()),
                            json: Some(data),
                        },
                        _ => Error::ApiError {
                            status: status,
                            description: None,
                            json: Some(data),
                        },
                    }
                } else {
                    Error::UnhandledStatus(res.status().as_u16())
                }
            }
            None => Error::UnhandledStatus(res.status().as_u16()),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Wrapping(Box::new(value))
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::Wrapping(Box::new(value))
    }
}

fn request_to_result<E, F, T>(req: RequestBuilder, f: F) -> Result<T>
where
    F: FnOnce(Response) -> std::result::Result<T, E>,
    Error: From<E>,
{
    let res = req.send()?;
    match res.status() {
        reqwest::StatusCode::OK => f(res).map_err(Error::from),
        reqwest::StatusCode::CREATED => f(res).map_err(Error::from),
        _ => Err(res.into()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use utilities::mocking;

    use super::*;

    #[test]
    fn it_can_parse_array_errors() {
        let _m = mocking::mock("GET", mocking::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(json!(["Oops!"]).to_string())
            .create();

        let res = reqwest::blocking::get(mocking::server_url()).unwrap();
        let error: Error = res.into();
        if let Error::ApiError {
            status,
            description,
            json: _,
        } = error
        {
            assert_eq!(status, 400);
            assert_eq!(description, Some("Oops!".into()));
        } else {
            dbg!(error);
            assert!(false);
        }
    }

    #[test]
    fn it_can_parse_detail_errors() {
        let _m = mocking::mock("GET", mocking::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"detail": "Not found"}).to_string())
            .create();

        let res = reqwest::blocking::get(mocking::server_url()).unwrap();
        let error: Error = res.into();
        if let Error::ApiError {
            status,
            description,
            json: _,
        } = error
        {
            assert_eq!(status, 404);
            assert_eq!(description, Some("Not found".into()));
        } else {
            dbg!(error);
            assert!(false);
        }
    }
}
