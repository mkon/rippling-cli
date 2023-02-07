use reqwest::{
    blocking::RequestBuilder,
    header::{HeaderMap, HeaderValue},
    Method,
};
use url::Url;

use super::Result;
use crate::persistence;

#[derive(Clone, Debug)]
pub struct Session {
    access_token: String,
    pub company: Option<String>,
    pub role: Option<String>,
    connection: reqwest::blocking::Client,
}

impl Session {
    pub fn new(token: String) -> Self {
        Self {
            access_token: token,
            company: None,
            role: None,
            connection: reqwest::blocking::Client::new(),
        }
    }

    pub fn load() -> Self {
        let state = persistence::State::load();
        Self {
            access_token: state.access_token.expect("State missing access token"),
            company: state.company_id,
            role: state.role_id,
            connection: reqwest::blocking::Client::new(),
        }
    }

    pub fn save(&self) {
        let state = persistence::State {
            access_token: Some(self.access_token.clone()),
            company_id: self.company.clone(),
            role_id: self.role.clone(),
        };
        state.store();
    }

    // pub fn company_and_role_set(&self) -> bool {
    //     [&self.company, &self.role].iter().all(|f| f.is_some())
    // }

    pub fn set_company_and_role(&mut self, company: String, role: String) {
        self.company = Some(company);
        self.role = Some(role);
    }

    pub fn company(&self) -> Option<&str> {
        self.company.as_ref().map(|s| s.as_str())
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_ref().map(|s| s.as_str())
    }

    pub fn get(&self, path: &str) -> Result<RequestBuilder> {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> Result<RequestBuilder> {
        self.request(Method::POST, path)
    }

    pub fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = self.url_for(&path)?;
        let rb = self
            .connection
            .request(method, url)
            .bearer_auth(&self.access_token)
            .headers(self.request_headers());
        Ok(rb)
    }

    fn request_headers(&self) -> HeaderMap {
        let mut map = HeaderMap::new();

        if let Some(value) = &self.company {
            map.append("company", HeaderValue::from_str(value).unwrap());
        }
        if let Some(value) = &self.role {
            map.append("role", HeaderValue::from_str(value).unwrap());
        }

        map
    }

    fn url_for(&self, path: &str) -> Result<Url> {
        #[cfg(not(test))]
        let url = "https://app.rippling.com/api/";
        #[cfg(test)]
        let url = &utilities::mocking::server_url();
        Ok(Url::parse(url)?.join(path)?)
    }
}
