use std::collections::HashMap;

use chrono::{DateTime, Local, Duration};
use reqwest::{
    blocking::RequestBuilder,
    header::{HeaderMap, HeaderValue},
    Method,
};
use serde::{Deserialize, Deserializer};
use serde_json::json;

use crate::persistance;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError {
        status: u16,
        data: HashMap<String, serde_json::Value>,
    },
    Wrapping(reqwest::Error),
    UnhandledStatus(u16),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        dbg!(self);
        match self {
            Self::ApiError { status, data } => {
                match data.get("detail") {
                    Some(string) => write!(f, "{string}"),
                    None => write!(f, "Unexpected response status {status}"),
                }
            },
            Self::UnhandledStatus(status) => write!(f, "Unexpected response status {status}"),
            Self::Wrapping(err) => write!(f, "{err}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
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

    pub fn tt_active_policy(&self) -> Result<TimeTrackPolicy> {
        let req = self
            .request_for(
                Method::GET,
                format!("https://app.rippling.com/api/time_tracking/api/time_entry_policies/get_active_policy"),
            );
        // let raw = res.text().unwrap();
        // println!("Response:\n{:?}", raw);
        let data = req.send()?.json::<HashMap<String, TimeTrackPolicy>>()?;
        Ok(data.values().next().unwrap().to_owned())
    }

    pub fn tt_break_policy(&self, id: &str) -> Result<TimeTrackBreakPolicy> {
        let req = self
            .request_for(
                Method::GET,
                format!("https://app.rippling.com/api/time_tracking/api/time_entry_break_policies/{id}"),
            );
        // let raw = res.text().unwrap();
        // println!("Response:\n{:?}", raw);
        Ok(req.send()?.json::<TimeTrackBreakPolicy>()?)
    }
    
    pub fn tt_break_end(&self, entry_id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .request_for(
                Method::POST,
                format!("https://app.rippling.com/api/time_tracking/api/time_entries/{entry_id}/end_break"),
            )
            .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
        dbg!(&req);
        // let raw = res.text().unwrap();
        // println!("Response:\n{:?}", raw);
        Ok(req.send()?.json::<TimeTrackEntry>()?)
    }

    pub fn tt_break_start(&self, entry_id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .request_for(
                Method::POST,
                format!("https://app.rippling.com/api/time_tracking/api/time_entries/{entry_id}/start_break"),
            )
            .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
        dbg!(&req);
        // let raw = res.text().unwrap();
        // println!("Response:\n{:?}", raw);
        Ok(req.send()?.json::<TimeTrackEntry>()?)
    }

    pub fn tt_clock_start(&self) -> Result<TimeTrackEntry> {
        let req = self
            .request_for(
                Method::POST,
                "https://app.rippling.com/api/time_tracking/api/time_entries/start_clock",
            )
            .json(&json!({"source": "WEB_CLOCK", "role": self.role.as_ref().unwrap()}));
        let res = req.send()?;
        match res.status() {
            reqwest::StatusCode::OK => Ok(res.json::<TimeTrackEntry>()?),
            _ => Err(Self::unexpected_status_error(res)),
        }
    }

    pub fn setup_company_and_role(&mut self) -> Result<()> {
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

    fn unexpected_status_error(res: reqwest::blocking::Response) -> Error {
        match res.headers().get("Content-Type") {
            Some(val) => {
                if val.to_str().unwrap().contains("application/json") {
                    Error::ApiError {
                        status: res.status().as_u16(),
                        data: res.json::<HashMap<String, serde_json::Value>>().unwrap()
                    }
                } else {
                    Error::UnhandledStatus(res.status().as_u16())
                }
            },
            None => { Error::UnhandledStatus(res.status().as_u16()) },
        }
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
    #[serde(rename = "regularHours", deserialize_with = "f32_from_str")]
    pub regular_hours: f32,
    // pub timezone: String,
}

fn f32_from_str<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer).unwrap();
    Ok(s.parse::<f32>().unwrap())
}

impl TimeTrackEntry {
    pub fn current_break(&self) -> Option<&TimeTrackEntryBreak> {
        self.breaks.iter().find(|b| b.end_time.is_none())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackPolicy {
    #[serde(rename = "breakPolicy")]
    pub break_policy_id: String,
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

impl TimeTrackEntryBreak {
    pub fn duration(&self) -> Option<Duration> {
        match self.end_time {
            Some(end) => Some(end - self.start_time),
            None => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackBreakPolicy {
    pub id: String,
    #[serde(rename = "companyBreakTypes")]
    pub break_types: Vec<TimeTrackBreakType>,
    #[serde(rename = "eligibleBreakTypes")]
    pub eligible_break_types: Vec<TimeTrackEligibleBreakType>,
}

impl TimeTrackBreakPolicy {
    pub fn manual_break_type(&self) -> Option<&TimeTrackBreakType> {
        let eligible_ids: Vec<&str> = self.eligible_break_types.iter().filter(|&bt| bt.allow_manual).map(|bt| bt.break_type_id.as_ref()).collect();
        self.break_types.iter().find(|bt| !bt.deleted && eligible_ids.contains(&&bt.id[..]))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackBreakType {
    pub id: String,
    #[serde(rename = "isDeleted")]
    pub deleted: bool,
    pub description: String,
    #[serde(rename = "minLength")]
    pub min_length: Option<f32>,
    #[serde(rename = "enforceMinLength")]
    pub enforce_min_length: bool,
    #[serde(rename = "maxLength")]
    pub max_length: Option<f32>,
    #[serde(rename = "enforceMaxLength")]
    pub enforce_max_length: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackEligibleBreakType {
    #[serde(rename = "allowManual")]
    allow_manual: bool,
    #[serde(rename = "breakType")]
    break_type_id: String,
}