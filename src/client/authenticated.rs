use chrono::{DateTime, Local};
use reqwest::{
    blocking::RequestBuilder,
    header::{HeaderMap, HeaderValue},
    Method,
};
use serde::{Deserialize};
use serde_json::json;

use crate::persistance;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    BadRequest(String),
    Wrapping(reqwest::Error),
    UnhandledStatus(u16),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        println!("reqwest error:");
        dbg!(&value);
        Error::Wrapping(value)
    }
}

pub struct Client {
    access_token: String,
    company: Option<String>,
    role: Option<String>,
    session: reqwest::blocking::Client,
}

impl Client {
    pub fn new(token: &str) -> Self {
        Self {
            access_token: token.to_owned(),
            company: None,
            role: None,
            session: reqwest::blocking::Client::new(),
        }
    }

    pub fn load() -> Self {
        let state = persistance::State::load();
        Self {
            access_token: state.access_token.expect("State missing access token"),
            company: state.company_id,
            role: state.role_id,
            session: reqwest::blocking::Client::new(),
        }
    }

    pub fn save(&self) {
        let state = persistance::State {
            access_token: Some(self.access_token.clone()),
            company_id: self.company.clone(),
            role_id: self.role.clone(),
        };
        state.store();
    }

    pub fn account_info(&self) -> Result<Vec<AccountInfo>> {
        let req = self.request_for(
            Method::GET,
            "https://app.rippling.com/api/auth_ext/get_account_info",
        );
        Ok(req.send()?.json::<Vec<AccountInfo>>()?)
    }

    pub fn tt_entries(&self) -> Result<Vec<TimeTrackEntry>> {
        let req = self
            .request_for(
                Method::GET,
                "https://app.rippling.com/api/time_tracking/api/time_entries/",
            )
            .query(&[("endTime", "")]); // Filter for entries with no end time
        Ok(req.send()?.json::<Vec<TimeTrackEntry>>()?)
    }


    pub fn tt_clock_start(&self) -> Result<TimeTrackEntry> {
        let req = self
            .request_for(
                Method::POST,
                "https://app.rippling.com/api/time_tracking/api/time_entries/start_clock",
            )
            .json(&json!({"source": "WEB_CLOCK", "role": self.role.as_ref().unwrap()}));
        dbg!(&req);
        let res = req.send()?;
        match res.status() {
            reqwest::StatusCode::ACCEPTED => Ok(res.json::<TimeTrackEntry>()?),
            reqwest::StatusCode::BAD_REQUEST => Err(Error::BadRequest(res.text()?)),
            _ => Err(Error::UnhandledStatus(res.status().as_u16())),
        }
    }

    pub fn setup_company_any_role(&mut self) -> Result<()> {
        if [&self.company, &self.role].iter().any(|f| f.is_none()) {
            let info = self.account_info()?;
            assert!(info.len() == 1, "Unexpected account info result");
            if let None = &self.company {
                self.company = Some(info[0].role.company.id.to_owned());
            }
            if let None = &self.role {
                self.role = Some(info[0].id.to_owned());
            }
        }
        Ok(())
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

    fn request_for<U>(&self, method: Method, url: U) -> RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        self.session
            .request(method, url)
            .bearer_auth(&self.access_token)
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
pub struct Oid {
    #[serde(rename = "$oid")]
    pub id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackEntry {
    pub id: String,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<Local>,
    #[serde(rename = "endTime")]
    pub end_time: Option<DateTime<Local>>,
    pub breaks: Vec<TimeTrackEntryBreak>,
    // pub timezone: String,
}

impl TimeTrackEntry {
    pub fn current_break(&self) -> Option<&TimeTrackEntryBreak> {
        self.breaks.iter().find(|b| b.end_time.is_none())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackEntryBreak {
    #[serde(rename = "companyBreakType")]
    pub break_type_id: String,
    pub description: String,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<Local>,
    #[serde(rename = "endTime")]
    pub end_time: Option<DateTime<Local>>,
}
