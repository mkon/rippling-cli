use chrono::{DateTime, Local};
use reqwest::{
    blocking::RequestBuilder,
    header::{HeaderMap, HeaderValue},
    Method,
};
use serde::Deserialize;

pub struct Client {
    id: String,
    secret: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
}

impl Client {
    pub fn new(id: &str, secret: &str) -> Self {
        Self {
            id: id.to_owned(),
            secret: secret.to_owned(),
            access_token: None,
            refresh_token: None,
            company: None,
            role: None,
        }
    }

    /// Returns tuple of access and refresh token
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<(String, String)> {
        let params = [
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ];
        let client = reqwest::blocking::Client::new();
        let req = client
            .post("https://app.rippling.com/api/o/token/")
            .form(&params)
            .basic_auth(&self.id, Some(&self.secret));
        let result: AuthResult = req.send()?.json()?;
        // println!("Response:\n{:?}", result);

        self.access_token = Some(result.access_token.clone());
        self.refresh_token = Some(result.refresh_token.clone());
        Ok((result.access_token.to_owned(), result.refresh_token.to_owned()))
    }

    pub fn account_info(&self) -> Result<Vec<AccountInfo>> {
        let req = self
            .authenticated_request_for(
                Method::GET,
                "https://app.rippling.com/api/auth_ext/get_account_info",
            );
        Ok(req.send()?.json::<Vec<AccountInfo>>()?)
    }

    pub fn current_user(&self) {
        let res = self
            .authenticated_request_for(
                Method::GET,
                "https://api.rippling.com/platform/api/me",
            )
            .send()
            .unwrap();
        let result = res.text().unwrap();
        println!("Response:\n{:?}", result);
    }

    pub fn tt_entries(&self) -> Result<Vec<TimeTrackEntry>> {
        let req = self
            .authenticated_request_for(
                Method::GET,
                "https://app.rippling.com/api/time_tracking/api/time_entries/",
            )
            .query(&[("endTime", "")]); // Filter for entries with no end time
        // let raw = res.text().unwrap();
        // println!("Response:\n{:?}", raw);
        Ok(req.send()?.json::<Vec<TimeTrackEntry>>()?)
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

    fn authenticated_request_for<U>(&self, method: Method, url: U) -> RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        reqwest::blocking::Client::new()
            .request(method, url)
            .bearer_auth(self.access_token.as_ref().unwrap())
            .headers(self.request_headers())
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub id: String,
    pub role: AccountInfoRole,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfoRole {
    pub company: Oid,
}

#[derive(Debug, Deserialize)]
pub struct AuthResult {
    pub access_token: String,
    // expires_in: u32,
    pub refresh_token: String,
    // token_type: String,
    // scope: String,
}

#[derive(Debug, Deserialize)]
pub struct Oid {
    #[serde(rename = "$oid")]
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeTrackEntry {
    pub id: String,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<Local>,
    #[serde(rename = "endTime")]
    pub end_time: Option<DateTime<Local>>,
    pub timezone: String,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {}

impl From<reqwest::Error> for Error {
    fn from(_value: reqwest::Error) -> Self {
        Error{}
    }
}