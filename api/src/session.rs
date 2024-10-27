use attohttpc::header::IntoHeaderName;
pub use attohttpc::Method;
pub use attohttpc::StatusCode;
use serde::de::DeserializeOwned;
use url::Url;

use super::Error;

/// Wraps the 3rd party http client RequestBuilder
pub struct Request {
    builder: attohttpc::RequestBuilder,
}

pub type Result<T> = std::result::Result<T, super::Error>;

impl Request {
    pub fn new(method: Method, url: Url) -> Self {
        let builder = attohttpc::RequestBuilder::new(method, url);
        Self { builder }
    }

    pub fn bearer_auth(self, token: &str) -> Self {
        Self { builder: self.builder.bearer_auth(token) }
    }

    pub fn header<K: IntoHeaderName>(self, key: K, value: String) -> Self {
        Self { builder: self.builder.header(key, value) }
    }

    pub fn param<K: AsRef<str>, V: ToString>(self, key: K, value: V) -> Self {
        Self { builder: self.builder.param(key, value) }
    }

    pub fn send(self) -> Result<Response> {
        Ok(Response::new(self.builder.send()?))
    }

    pub fn send_json<J: serde::Serialize>(self, json: J) -> Result<Response> {
        Ok(Response::new(self.builder.json(&json)?.send()?))
    }
}

pub struct Response {
    response: attohttpc::Response,
    parse_states: Vec<StatusCode>,
}

impl Response {
    pub fn new(response: attohttpc::Response) -> Self {
        Self { response, parse_states: vec![StatusCode::OK, StatusCode::CREATED] }
    }

    pub fn accept_states(self, states: Vec<StatusCode>) -> Self {
        Self { response: self.response, parse_states: states }
    }

    pub fn into_error(self) -> Error {
        <Error as From<attohttpc::Response>>::from(self.response)
    }

    pub fn parse_json<J>(self) -> Result<J>
    where
        J: DeserializeOwned,
    {
        let res = self.response;
        if self.parse_states.contains(&res.status()) {
            Ok(res.json::<J>()?)
        } else {
            Err(res.into())
        }
    }

    pub fn status(&self) -> StatusCode {
        self.response.status()
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub access_token: String,
    pub company: Option<String>,
    pub role: Option<String>,
    pub url: Url,
}

impl Session {
    pub fn company(&self) -> Option<&str> {
        self.company.as_ref().map(|s| s.as_str())
    }

    pub fn get(&self, path: &str) -> Request {
        self.request(Method::GET, path)
    }

    pub fn get_json<J: DeserializeOwned>(&self, path: &str) -> Result<J> {
        self.request(Method::GET, path).send()?.parse_json::<J>()
    }

    pub fn new(url: Option<Url>, token: String) -> Self {
        let url = url.unwrap_or(super::default_root());
        Self { access_token: token, company: None, role: None, url }
    }

    pub fn post(&self, path: &str) -> Request {
        self.request(Method::POST, path)
    }

    fn request(&self, method: Method, path: &str) -> Request {
        let url = self.url.join(path).unwrap();
        let mut builder = Request::new(method, url).bearer_auth(&self.access_token);
        if let Some(value) = &self.company {
            builder = builder.header("company", value.to_owned());
        }
        if let Some(value) = &self.role {
            builder = builder.header("role", value.to_owned());
        }
        builder
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_ref().map(|s| s.as_str())
    }

    pub fn set_company_and_role(&mut self, company: String, role: String) {
        self.company = Some(company);
        self.role = Some(role);
    }
}

#[cfg(test)]
/// Helper function for DRY tests
pub fn test_session(server: &utilities::mocking::FakeRippling) -> Session {
    let url = Url::parse(&server.url()).unwrap();
    Session {
        access_token: "access-token".into(),
        company: Some("some-company-id".into()),
        role: Some("some-role-id".into()),
        url,
    }
}
