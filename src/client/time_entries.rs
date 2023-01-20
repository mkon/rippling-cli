use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Deserializer};
use serde_json::json;

use super::session::Session;
use super::Result;

pub fn current_entry(session: &Session) -> Result<Option<TimeTrackEntry>> {
    let req = session.get("time_tracking/api/time_entries")?.query(&[("endTime", "")]); // Filter for entries with no end time
    super::request_to_result(req, |r| {
        let entries = r.json::<Vec<TimeTrackEntry>>()?;
        Result::Ok(entries.into_iter().next())
    })
}

pub fn start_break(session: &Session, id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
    let req = session
        .post(&format!("time_tracking/api/time_entries/{id}/end_break"))?
        .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
    super::request_to_result(req, |r| r.json::<TimeTrackEntry>())
}

pub fn end_break(session: &Session, id: &str, break_type_id: &str) -> Result<TimeTrackEntry> {
    let req = session
        .post(&format!("time_tracking/api/time_entries/{id}/start_break"))?
        .json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}));
    super::request_to_result(req, |r| r.json::<TimeTrackEntry>())
}

pub fn start_clock(session: &Session) -> Result<TimeTrackEntry> {
    let req = session
        .post("time_tracking/api/time_entries/start_clock")?
        .json(&json!({"source": "WEB_CLOCK", "role": session.role().unwrap()}));
    super::request_to_result(req, |r| r.json::<TimeTrackEntry>())
}

pub fn end_clock(session: &Session, id: &str) -> Result<TimeTrackEntry> {
    let req = session
        .post(&format!("time_tracking/api/time_entries/{id}/stop_clock"))?
        .json(&json!({"source": "WEB_CLOCK"}));
    super::request_to_result(req, |r| r.json::<TimeTrackEntry>())
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

impl TimeTrackEntry {
    pub fn current_break(&self) -> Option<&TimeTrackEntryBreak> {
        self.breaks.iter().find(|b| b.end_time.is_none())
    }
}

impl TimeTrackEntryBreak {
    pub fn duration(&self) -> Option<Duration> {
        match self.end_time {
            Some(end) => Some(end - self.start_time),
            None => None,
        }
    }
}

fn f32_from_str<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer).unwrap();
    Ok(s.parse::<f32>().unwrap())
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

    fn session() -> Session {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("some-company-id".into(), "some-role-id".into());
        session
    }

    #[test]
    fn it_can_fetch_current_entry() {
        let _m = mock_api("GET", "/time_tracking/api/time_entries?endTime=", "time_entries").create();

        let entry = current_entry(&session()).unwrap().unwrap();
        assert_eq!(entry.active_policy.break_policy_id, "some-break-policy");
        assert_eq!(entry.start_time.with_timezone(&Utc).to_rfc3339(), "2023-01-19T08:22:25+00:00");
        assert_eq!(entry.regular_hours, 0.92583334);
        assert!(entry.current_break().is_none());
    }

    #[test]
    fn it_can_start_the_clock() {
        let _m = mock_api("POST", "/time_tracking/api/time_entries/start_clock", "time_entry")
            .match_body(Matcher::Json(json!({"source": "WEB_CLOCK", "role": "some-role-id"})))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        let entry = start_clock(&session()).unwrap();
        assert_eq!(entry.start_time.with_timezone(&Utc).to_rfc3339(), "2023-01-19T08:22:25+00:00");
    }

    #[test]
    fn it_can_stop_the_clock() {
        let _m = mock_api("POST", "/time_tracking/api/time_entries/id/stop_clock", "time_entry")
            .match_body(Matcher::Json(json!({"source": "WEB_CLOCK"})))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        let entry = end_clock(&session(), &"id").unwrap();
        assert_eq!(entry.start_time.with_timezone(&Utc).to_rfc3339(), "2023-01-19T08:22:25+00:00");
    }
}
