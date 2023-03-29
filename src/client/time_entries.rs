use json_value_merge::Merge;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use time::{Duration, OffsetDateTime};

use time::serde::rfc3339;

use super::session::Session;
use super::Result;

pub fn create_entry(session: &Session, entry: &NewTimeEntry) -> Result<TimeEntry> {
    let mut body = json!(&entry);
    body.merge(json!({"company":session.company(), "role":session.role()}));
    session
        .post("time_tracking/api/time_entries")
        .send_json(&body)?
        .parse_json()
}

pub fn current_entry(session: &Session) -> Result<Option<TimeEntry>> {
    let entries: Vec<TimeEntry> = session
        .get("time_tracking/api/time_entries")
        .param("endTime", "") // Filter for entries with no end time
        .send()?
        .parse_json()?;
    Result::Ok(entries.into_iter().next())
}

pub fn start_break(session: &Session, id: &str, break_type_id: &str) -> Result<TimeEntry> {
    session
        .post(&format!("time_tracking/api/time_entries/{id}/start_break"))
        .send_json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}))?
        .parse_json()
}

pub fn end_break(session: &Session, id: &str, break_type_id: &str) -> Result<TimeEntry> {
    session
        .post(&format!("time_tracking/api/time_entries/{id}/end_break"))
        .send_json(&json!({"source": "WEB_CLOCK", "break_type": break_type_id}))?
        .parse_json()
}

pub fn start_clock(session: &Session) -> Result<TimeEntry> {
    session
        .post("time_tracking/api/time_entries/start_clock")
        .send_json(&json!({"source": "WEB_CLOCK", "role": session.role().unwrap()}))?
        .parse_json()
}

pub fn end_clock(session: &Session, id: &str) -> Result<TimeEntry> {
    session
        .post(&format!("time_tracking/api/time_entries/{id}/stop_clock"))
        .send_json(&json!({"source": "WEB_CLOCK"}))?
        .parse_json()
}

#[derive(Clone, Debug, Serialize)]
pub struct NewTimeEntry {
    #[serde(rename = "jobShifts")]
    pub shifts: Vec<NewTimeEntryShift>,
    pub breaks: Vec<NewTimeEntryBreak>,
    source: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NewTimeEntryBreak {
    #[serde(rename = "companyBreakType")]
    pub break_type_id: String,
    #[serde(rename = "startTime", with = "rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(rename = "endTime", with = "rfc3339")]
    pub end_time: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize)]
pub struct NewTimeEntryShift {
    #[serde(rename = "startTime", with = "rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(rename = "endTime", with = "rfc3339")]
    pub end_time: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    #[serde(rename = "activePolicy")]
    pub active_policy: TimeEntryActivePolicy,
    #[serde(rename = "startTime", with = "rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(rename = "endTime", with = "rfc3339::option")]
    pub end_time: Option<OffsetDateTime>,
    pub breaks: Vec<TimeEntryBreak>,
    #[serde(rename = "regularHours", deserialize_with = "f32_from_str")]
    pub regular_hours: f32,
    #[serde(rename = "unpaidBreakHours", deserialize_with = "f32_from_str")]
    pub unpaid_break_hours: f32,
    // pub timezone: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeEntryActivePolicy {
    #[serde(rename = "timePolicy")]
    pub time_policy_id: String,
    #[serde(rename = "breakPolicy")]
    pub break_policy_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeEntryBreak {
    #[serde(rename = "companyBreakType")]
    pub break_type_id: String,
    pub description: String,
    #[serde(rename = "startTime", with = "rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(rename = "endTime", with = "rfc3339::option")]
    pub end_time: Option<OffsetDateTime>,
}

impl NewTimeEntry {
    pub fn new() -> Self {
        Self { shifts: Vec::new(), breaks: Vec::new(), source: "WEB".into() }
    }

    pub fn add_shift(&mut self, start_time: OffsetDateTime, end_time: OffsetDateTime) {
        self.shifts
            .push(NewTimeEntryShift { start_time: start_time, end_time: end_time });
    }

    pub fn add_break(&mut self, break_type: String, start_time: OffsetDateTime, end_time: OffsetDateTime) {
        self.breaks
            .push(NewTimeEntryBreak { break_type_id: break_type, start_time: start_time, end_time: end_time });
    }
}

impl TimeEntry {
    pub fn current_break(&self) -> Option<&TimeEntryBreak> {
        self.breaks.iter().find(|b| b.end_time.is_none())
    }
}

impl TimeEntryBreak {
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
    use super::*;
    use time::{format_description::well_known::Rfc3339, macros::datetime, UtcOffset};
    use utilities::mocking;

    fn session() -> Session {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("some-company-id".into(), "some-role-id".into());
        session
    }

    #[test]
    fn it_can_create_entries() {
        let mut new_entry = NewTimeEntry::new();
        new_entry.add_shift(datetime!(2023-01-20 08:00 +1), datetime!(2023-01-20 17:00 +1));
        new_entry.add_break(
            "some-break-type".into(),
            datetime!(2023-01-20 12:00 +1),
            datetime!(2023-01-20 12:45 +1),
        );

        let m = mocking::with_fixture("POST", "/time_tracking/api/time_entries", "time_entry")
            .with_status(201)
            .match_body(mocking::Matcher::Json(json!(
                {
                    "jobShifts": [
                        {
                            "startTime": "2023-01-20T08:00:00+01:00",
                            "endTime": "2023-01-20T17:00:00+01:00"
                        }
                    ],
                    "breaks": [
                        {
                            "companyBreakType": "some-break-type",
                            "startTime": "2023-01-20T12:00:00+01:00",
                            "endTime": "2023-01-20T12:45:00+01:00"
                        }
                    ],
                    "company": "some-company-id",
                    "role": "some-role-id",
                    "source": "WEB"
                }
            )))
            .create();

        let entry = create_entry(&session(), &new_entry);
        assert!(entry.is_ok());
        m.assert();
    }

    #[test]
    fn it_can_fetch_current_entry() {
        let _m = mocking::with_fixture("GET", "/time_tracking/api/time_entries?endTime=", "time_entries").create();

        let entry = current_entry(&session()).unwrap().unwrap();
        assert_eq!(entry.active_policy.break_policy_id, "some-break-policy");
        assert_eq!(
            entry.start_time.to_offset(UtcOffset::UTC).format(&Rfc3339).unwrap(),
            "2023-01-19T08:22:25Z"
        );
        assert_eq!(entry.regular_hours, 0.92583334);
        assert!(entry.current_break().is_none());
    }

    #[test]
    fn it_can_start_the_clock() {
        let _m = mocking::with_fixture("POST", "/time_tracking/api/time_entries/start_clock", "time_entry")
            .match_body(mocking::Matcher::Json(
                json!({"source": "WEB_CLOCK", "role": "some-role-id"}),
            ))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        let entry = start_clock(&session()).unwrap();
        assert_eq!(
            entry.start_time.to_offset(UtcOffset::UTC).format(&Rfc3339).unwrap(),
            "2023-01-19T08:22:25Z"
        );
    }

    #[test]
    fn it_can_stop_the_clock() {
        let _m = mocking::with_fixture("POST", "/time_tracking/api/time_entries/id/stop_clock", "time_entry")
            .match_body(mocking::Matcher::Json(json!({"source": "WEB_CLOCK"})))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        let entry = end_clock(&session(), &"id").unwrap();
        assert_eq!(
            entry.start_time.to_offset(UtcOffset::UTC).format(&Rfc3339).unwrap(),
            "2023-01-19T08:22:25Z"
        );
    }

    #[test]
    fn it_can_take_a_break() {
        let m = mocking::with_fixture("POST", "/time_tracking/api/time_entries/id/start_break", "time_entry")
            .match_body(mocking::Matcher::Json(
                json!({"source": "WEB_CLOCK", "break_type": "break-type-id"}),
            ))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        start_break(&session(), &"id", &"break-type-id").unwrap();
        m.assert()
    }

    #[test]
    fn it_can_stop_a_break() {
        let m = mocking::with_fixture("POST", "/time_tracking/api/time_entries/id/end_break", "time_entry")
            .match_body(mocking::Matcher::Json(
                json!({"source": "WEB_CLOCK", "break_type": "break-type-id"}),
            ))
            .match_header("company", "some-company-id")
            .match_header("role", "some-role-id")
            .create();

        end_break(&session(), &"id", &"break-type-id").unwrap();
        m.assert()
    }
}
