#[derive(Debug)]
pub enum Error {
    ApiError {
        status: u16,
        description: Option<String>,
        json: Option<serde_json::Value>,
    },
    Generic(String),
    UnexpectedPayload,
    UnhandledStatus(u16),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError { status, description, json: _ } => match description {
                Some(string) => write!(f, "{string}"),
                None => write!(f, "Unexpected response status {status}"),
            },
            Self::UnexpectedPayload => write!(f, "Unexpected account info response"),
            Self::UnhandledStatus(status) => write!(f, "Unexpected response status {status}"),
            Self::Generic(err) => write!(f, "{err}"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Generic(format!("{value}"))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Generic(format!("{value}"))
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::Generic(format!("{value}"))
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        let desc = format!("{value}");
        match value.into_response() {
            Some(res) => match res.header("Content-Type") {
                Some(val) => {
                    if val.contains("application/json") {
                        let status = res.status();
                        let data = res.into_json::<serde_json::Value>().unwrap();
                        match &data {
                            serde_json::Value::Array(list) if list.first().unwrap().is_string() => Error::ApiError {
                                status,
                                description: list.first().map(|v| v.as_str().unwrap().to_owned()),
                                json: Some(data),
                            },
                            serde_json::Value::Object(obj) if obj.contains_key("detail") => Error::ApiError {
                                status,
                                description: obj["detail"].as_str().map(std::borrow::ToOwned::to_owned),
                                json: Some(data),
                            },
                            _ => Error::ApiError { status, description: None, json: Some(data) },
                        }
                    } else {
                        Error::UnhandledStatus(res.status())
                    }
                }
                None => Error::UnhandledStatus(res.status()),
            },
            None => Error::Generic(desc),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use utilities::mocking;

    use super::*;

    #[test]
    fn it_can_parse_array_errors() {
        let mut server = mocking::FakeRippling::new();
        let _m = server
            .mock("GET", mocking::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(json!(["Oops!"]).to_string())
            .create();

        let req = ureq::get(&server.url()).call();
        match req {
            Ok(ok) => {
                dbg!(ok);
                assert!(false);
            }
            Err(error) => {
                let error: crate::Error = error.into();
                match error {
                    Error::ApiError { status, description, json: _ } => {
                        assert_eq!(status, 400);
                        assert_eq!(description, Some("Oops!".into()));
                    }
                    _ => assert!(false),
                }
            }
        }
    }

    #[test]
    fn it_can_parse_detail_errors() {
        let mut server = mocking::FakeRippling::new();
        let _m = server
            .mock("GET", mocking::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"detail": "Not found"}).to_string())
            .create();

        let req = ureq::get(&server.url()).call();
        match req {
            Ok(ok) => {
                dbg!(ok);
                assert!(false);
            }
            Err(error) => {
                let error: crate::Error = error.into();
                match error {
                    Error::ApiError { status, description, json: _ } => {
                        assert_eq!(status, 404);
                        assert_eq!(description, Some("Not found".into()));
                    }
                    _ => assert!(false),
                }
            }
        }
    }
}
