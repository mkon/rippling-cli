use std::{collections::HashMap, result::Result as StdResult};

use chrono::{DateTime, Duration, Local};
use reqwest::blocking::{RequestBuilder, Response};
use reqwest::Method;
use serde::{Deserialize, Deserializer};
use serde_json::json;

#[cfg(test)]
use mockito;

use super::session::Session;
use super::{Error, Result};

pub struct Client {
    session: Session,
}

impl Client {
    pub fn new(session: Session) -> Self {
        Self { session: session }
    }

    pub fn load() -> Self {
        Self::new(Session::load())
    }

    pub fn save(&self) {
        self.session.save();
    }

    pub fn account_info(&self) -> Result<AccountInfo> {
        let req = self.get("auth_ext/get_account_info")?;
        request_to_result(req, |r| {
            let list = r.json::<Vec<AccountInfo>>()?;
            list.into_iter().next().ok_or(Error::UnexpectedPayload)
        })
    }

    pub fn tt_current_entry(&self) -> Result<Option<TimeTrackEntry>> {
        let req = self.get("time_tracking/api/time_entries")?.query(&[("endTime", "")]); // Filter for entries with no end time
        request_to_result(req, |r| {
            let entries = r.json::<Vec<TimeTrackEntry>>()?;
            Result::Ok(entries.into_iter().next())
        })
    }

    pub fn tt_active_policy(&self) -> Result<TimeTrackPolicy> {
        let req = self.get("time_tracking/api/time_entry_policies/get_active_policy")?;
        request_to_result(req, |r| {
            let policies = r.json::<HashMap<String, TimeTrackPolicy>>()?;
            policies.into_iter().map(|(_, v)| v).next().ok_or(Error::MissingActivePolicy)
        })
    }

    pub fn tt_break_policy(&self, id: &str) -> Result<TimeTrackBreakPolicy> {
        let req = self.get(&format!("time_tracking/api/time_entry_break_policies/{id}"))?;
        request_to_result(req, |r| r.json::<TimeTrackBreakPolicy>())
    }

    pub fn tt_break_end(&self, id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .post(&format!("time_tracking/api/time_entries/{id}/end_break"))?
            .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn tt_break_start(&self, id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .post(&format!("time_tracking/api/time_entries/{id}/start_break"))?
            .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn tt_clock_start(&self) -> Result<TimeTrackEntry> {
        let req = self
            .post("time_tracking/api/time_entries/start_clock")?
            .json(&json!({"source": "WEB_CLOCK", "role": self.session.role().unwrap()}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn tt_clock_stop(&self, id: &str) -> Result<TimeTrackEntry> {
        let req = self
            .post(&format!("time_tracking/api/time_entries/{id}/stop_clock"))?
            .json(&json!({"source": "WEB_CLOCK"}));
        request_to_result(req, |r| r.json::<TimeTrackEntry>())
    }

    pub fn setup_company_and_role(&mut self) -> Result<()> {
        if !self.session.company_and_role_set() {
            let info = self.account_info()?;
            self.session.set_company_and_role(info.role.company.id, info.id);
        }
        Ok(())
    }

    fn get(&self, path: &str) -> Result<RequestBuilder> {
        self.session.request(Method::GET, path)
    }

    fn post(&self, path: &str) -> Result<RequestBuilder> {
        self.session.request(Method::POST, path)
    }
}

fn request_to_result<E, F, T>(req: RequestBuilder, f: F) -> Result<T>
where
    F: FnOnce(Response) -> StdResult<T, E>,
    Error: From<E>,
{
    // dbg!(&req);
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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mockito::{mock, Matcher};

    use super::*;

    fn mock_api(method: &str, path: &str, fixture: &str) -> mockito::Mock {
        mock(method, path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_file(format!("tests/fixtures/{fixture}.json"))
            .match_header("authorization", "Bearer access-token")
    }

    fn client() -> Client {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("some-company-id".into(), "some-role-id".into());
        Client::new(session)
    }

    #[test]
    fn it_can_fetch_account_info() {
        let _m = mock_api("GET", "/auth_ext/get_account_info", "account_info").create();

        let info = client().account_info().unwrap();
        assert_eq!(info.role.company.id, "some-company-id");
        assert_eq!(info.id, "some-role-id");
    }

    #[test]
    fn it_can_fetch_current_entry() {
        let _m = mock_api("GET", "/time_tracking/api/time_entries?endTime=", "time_entries").create();

        let entry = client().tt_current_entry().unwrap().unwrap();
        assert_eq!(entry.active_policy.break_policy_id, "some-break-policy");
        assert_eq!(entry.start_time.with_timezone(&Utc).to_rfc3339(), "2023-01-19T08:22:25+00:00");
        assert_eq!(entry.regular_hours, 0.92583334);
        assert!(entry.current_break().is_none());
    }

    #[test]
    fn it_can_fetch_a_break_policy() {
        let _m = mock_api("GET", "/time_tracking/api/time_entry_break_policies/policy-id", "break_policy").create();

        let policy = client().tt_break_policy("policy-id").unwrap();
        let mybreak = policy.manual_break_type().unwrap();
        assert_eq!(mybreak.id, "break-id-1");
        assert_eq!(mybreak.description, "Lunch Break - Manually clock in/out");
    }

    #[test]
    fn it_can_start_the_clock() {
        let _m = mock_api("POST", "/time_tracking/api/time_entries/start_clock", "time_entry")
            .match_body(Matcher::Json(json!({"source": "WEB_CLOCK", "role": "some-role-id"})))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        let entry = client().tt_clock_start().unwrap();
        assert_eq!(entry.start_time.with_timezone(&Utc).to_rfc3339(), "2023-01-19T08:22:25+00:00");
    }

    #[test]
    fn it_can_stop_the_clock() {
        let _m = mock_api("POST", "/time_tracking/api/time_entries/id/stop_clock", "time_entry")
            .match_body(Matcher::Json(json!({"source": "WEB_CLOCK"})))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        let entry = client().tt_clock_stop(&"id").unwrap();
        assert_eq!(entry.start_time.with_timezone(&Utc).to_rfc3339(), "2023-01-19T08:22:25+00:00");
    }
}
