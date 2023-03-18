pub mod account_info;
pub mod break_policy;
pub mod mfa;
pub mod pto;
mod public;
mod session;
pub mod time_entries;

pub use public::Client as PublicClient;
use attohttpc::RequestBuilder;
use attohttpc::Response;
pub use session::Session;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError {
        status: u16,
        description: Option<String>,
        json: Option<serde_json::Value>,
    },
    Wrapping(String),
    UnexpectedPayload,
    UnhandledStatus(u16),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl From<attohttpc::Error> for Error {
    fn from(value: attohttpc::Error) -> Self {
        Error::Wrapping(format!("{}", value))
    }
}

impl From<Response> for Error {
    fn from(res: Response) -> Self {
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
        Error::Wrapping(format!("{}", value))
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::Wrapping(format!("{}", value))
    }
}

fn request_to_result<E, F, T, B>(req: RequestBuilder<B>, f: F) -> Result<T>
where
    F: FnOnce(Response) -> std::result::Result<T, E>,
    Error: From<E>,
    B: attohttpc::body::Body
{
    let res = req.send()?;
    match res.status() {
        attohttpc::StatusCode::OK => f(res).map_err(Error::from),
        attohttpc::StatusCode::CREATED => f(res).map_err(Error::from),
        _ => {
            dbg!("Not a ok response");
            Err(res.into())
        },
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

        let req = attohttpc::get(mocking::server_url());
        let error: Error = req.send().unwrap().into();
        if let Error::ApiError { status, description, json: _ } = error {
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

        let req = attohttpc::get(mocking::server_url());
        let error: Error = req.send().unwrap().into();
        if let Error::ApiError { status, description, json: _ } = error {
            assert_eq!(status, 404);
            assert_eq!(description, Some("Not found".into()));
        } else {
            dbg!(error);
            assert!(false);
        }
    }
}
