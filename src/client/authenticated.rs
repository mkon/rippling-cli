use std::{collections::HashMap, result::Result as StdResult};

use chrono::{DateTime, Duration, Local};
use reqwest::{
    blocking::{RequestBuilder, Response},
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use url::Url;

#[cfg(test)]
use mockito;

use crate::persistence;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ApiError {
        status: u16,
        data: HashMap<String, serde_json::Value>,
    },
    MissingActivePolicy,
    Wrapping(Box<dyn std::error::Error>),
    UnexpectedAccountInfo,
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
            Self::UnexpectedAccountInfo => write!(f, "Unexpected account info response"),
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
        let state = persistence::State::load();
        Self {
            access_token: state.access_token.expect("State missing access token"),
            company: state.company_id,
            role: state.role_id,
            session: reqwest::blocking::Client::new(),
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

    pub fn account_info(&self) -> Result<AccountInfo> {
        let req = self.get("auth_ext/get_account_info");
        request_to_result(req, |r| {
            let list = r.json::<Vec<AccountInfo>>()?;
            list.into_iter().next().ok_or(Error::UnexpectedAccountInfo)
        })
    }

    pub fn tt_current_entry(&self) -> Result<Option<TimeTrackEntry>> {
        let req = self
            .get("time_tracking/api/time_entries")
            .query(&[("endTime", "")]); // Filter for entries with no end time
        request_to_result(req, |r| {
            let entries = r.json::<Vec<TimeTrackEntry>>()?;
            Result::Ok(entries.into_iter().next())
        })
    }

    pub fn tt_active_policy(&self) -> Result<TimeTrackPolicy> {
        let req = self.get("time_tracking/api/time_entry_policies/get_active_policy");
        request_to_result(req, |r| {
            let policies = r.json::<HashMap<String, TimeTrackPolicy>>()?;
            policies
                .into_iter()
                .map(|(_, v)| v)
                .next()
                .ok_or(Error::MissingActivePolicy)
        })
    }

    pub fn tt_break_policy(&self, id: &str) -> Result<TimeTrackBreakPolicy> {
        let req = self.get(&format!("time_tracking/api/time_entry_break_policies/{id}"));
        request_to_result(req, |r| r.json::<TimeTrackBreakPolicy>())
    }

    pub fn tt_break_end(&self, id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .post(&format!("time_tracking/api/time_entries/{id}/end_break"))
            .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn tt_break_start(&self, id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .post(&format!("time_tracking/api/time_entries/{id}/start_break"))
            .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn tt_clock_start(&self) -> Result<TimeTrackEntry> {
        let req = self
            .post("time_tracking/api/time_entries/start_clock")
            .json(&json!({"source": "WEB_CLOCK", "role": self.role.as_ref().unwrap()}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn tt_clock_stop(&self, id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .post(&format!("time_tracking/api/time_entries/{id}/stop_clock"))
            .json(&json!({"source": "WEB_CLOCK"}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn setup_company_and_role(&mut self) -> Result<()> {
        if [&self.company, &self.role].iter().any(|f| f.is_none()) {
            let info = self.account_info()?;
            if let None = &self.company {
                self.company = Some(info.role.company.id.to_owned());
            }
            if let None = &self.role {
                self.role = Some(info.id.to_owned());
            }
        }
        Ok(())
    }

    fn get(&self, path: &str) -> RequestBuilder {
        self.session
            .get(self.url_for(&path).unwrap())
            .bearer_auth(&self.access_token)
            .headers(self.request_headers())
    }

    fn post(&self, path: &str) -> RequestBuilder {
        self.session
            .post(self.url_for(&path).unwrap())
            .bearer_auth(&self.access_token)
            .headers(self.request_headers())
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
        let url = &mockito::server_url();
        Ok(Url::parse(url)?.join(path)?)
    }
}

fn request_to_result<E, F, T>(req: RequestBuilder, f: F) -> Result<T>
where
    F: FnOnce(Response) -> StdResult<T, E>,
    Error: From<E>,
{
    let res = req.send()?;
    // dbg!(&res);
    match res.status() {
        reqwest::StatusCode::OK => f(res).map_err(Error::from),
        _ => Err(res.into()),
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
    #[serde(rename = "activePolicy")]
    pub active_policy: TimeTrackActivePolicy,
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
where
    D: Deserializer<'de>,
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
pub struct TimeTrackActivePolicy {
    #[serde(rename = "timePolicy")]
    pub time_policy_id: String,
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
pub struct TimeTrackPolicy {
    #[serde(rename = "breakPolicy")]
    pub break_policy_id: String,
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
        let eligible_ids: Vec<&str> = self
            .eligible_break_types
            .iter()
            .filter(|&bt| bt.allow_manual)
            .map(|bt| bt.break_type_id.as_ref())
            .collect();
        self.break_types
            .iter()
            .find(|bt| !bt.deleted && eligible_ids.contains(&&bt.id[..]))
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

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::Utc;
    use mockito::mock;

    use super::*;

    fn mock_api(method: &str, path: &str, fixture: &str) -> mockito::Mock {
        mock(method, path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fs::read_to_string(format!("tests/fixtures/{fixture}.json")).unwrap())
            .create()
    }

    #[test]
    fn it_can_fetch_account_info() {
        let _m = mock_api("GET", "/auth_ext/get_account_info", "account_info");

        let client = Client::new("access-token");
        let info = client.account_info().unwrap();
        assert_eq!(info.role.company.id, "some-company-id");
        assert_eq!(info.id, "some-role-id");
    }

    #[test]
    fn it_can_fetch_current_entry() {
        let _m = mock_api("GET", "/time_tracking/api/time_entries?endTime=", "time_entries");

        let client = Client::new("access-token");
        let entry = client.tt_current_entry().unwrap().unwrap();
        assert_eq!(entry.active_policy.break_policy_id, "some-break-policy");
        assert_eq!(
            entry.start_time.with_timezone(&Utc).to_rfc3339(),
            "2023-01-19T08:22:25+00:00"
        );
        assert_eq!(entry.regular_hours, 0.92583334);
        assert!(entry.current_break().is_none());
    }

    #[test]
    fn it_can_fetch_a_break_policy() {
        let _m = mock_api("GET", "/time_tracking/api/time_entry_break_policies/policy-id", "break_policy");

        let client = Client::new("access-token");
        let policy = client.tt_break_policy("policy-id").unwrap();
        let mybreak = policy.manual_break_type().unwrap();
        assert_eq!(mybreak.id, "break-id-1");
        assert_eq!(mybreak.description, "Lunch Break - Manually clock in/out");
    }
}
