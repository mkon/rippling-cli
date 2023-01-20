use serde::Deserialize;

use super::session::Session;
use super::Result;

pub fn fetch(session: &Session, id: &str) -> Result<TimeTrackBreakPolicy> {
    let req = session.get(&format!("time_tracking/api/time_entry_break_policies/{id}"))?;
    super::request_to_result(req, |r| r.json::<TimeTrackBreakPolicy>())
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackBreakPolicy {
    pub id: String,
    #[serde(rename = "companyBreakTypes")]
    pub break_types: Vec<TimeTrackBreakType>,
    #[serde(rename = "eligibleBreakTypes")]
    pub eligible_break_types: Vec<TimeTrackEligibleBreakType>,
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

#[cfg(test)]
mod tests {
    use mockito::mock;

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
    fn it_can_fetch_a_break_policy() {
        let _m = mock_api("GET", "/time_tracking/api/time_entry_break_policies/policy-id", "break_policy").create();

        let policy = fetch(&session(), "policy-id").unwrap();
        let mybreak = policy.manual_break_type().unwrap();
        assert_eq!(mybreak.id, "break-id-1");
        assert_eq!(mybreak.description, "Lunch Break - Manually clock in/out");
    }
}
