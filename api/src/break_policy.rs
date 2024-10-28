use std::collections::HashMap;

use serde::Deserialize;

use super::{Error, Result};

impl super::Client {
    pub fn active_break_policy(&self) -> Result<ActivePolicy> {
        let mut map: HashMap<String, ActivePolicy> = self
            .get("time_tracking/api/time_entry_policies/get_active_policy")
            .call()?
            .into_json()?;
        map.remove(self.role().unwrap()).ok_or(Error::UnexpectedPayload)
    }

    pub fn break_policy(&self, id: &str) -> Result<BreakPolicy> {
        let break_policy: BreakPolicy = self
            .get(&format!("time_tracking/api/time_entry_break_policies/{id}"))
            .call()?
            .into_json()?;
        Ok(break_policy)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActivePolicy {
    #[serde(rename = "timePolicy")]
    pub time_policy: String,
    #[serde(rename = "breakPolicy")]
    pub break_policy: String,
    #[serde(rename = "roleOverrides")]
    pub role_overrides: RoleOverrides,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BreakPolicy {
    pub id: String,
    #[serde(rename = "companyBreakTypes")]
    pub break_types: Vec<BreakType>,
    #[serde(rename = "eligibleBreakTypes")]
    pub eligible_break_types: Vec<EligibleBreakType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BreakType {
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
pub struct EligibleBreakType {
    #[serde(rename = "allowManual")]
    allow_manual: bool,
    #[serde(rename = "breakType")]
    break_type_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RoleOverrides {
    #[serde(rename = "roleProperties")]
    pub role_properties: RoleProperties,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RoleProperties {
    pub role: String,
    #[serde(rename = "defaultTimezone")]
    pub default_timezone: String,
}

impl BreakPolicy {
    pub fn manual_break_type(&self) -> Option<&BreakType> {
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

#[cfg(test)]
mod tests {
    use utilities::mocking;

    use crate::Client;

    fn setup() -> (mocking::FakeRippling, Client) {
        let server = mocking::FakeRippling::new();
        let client = Client::new("access-token".to_owned())
            .with_root(url::Url::parse(&server.url()).unwrap())
            .with_company_and_role("some-company-id".to_owned(), "some-role-id".to_owned());
        (server, client)
    }

    #[test]
    fn it_can_fetch_a_break_policy() {
        let (mut server, client) = setup();
        let _m = server.mock_break_policy("policy-id");

        let policy = client.break_policy("policy-id").unwrap();
        let mybreak = policy.manual_break_type().unwrap();
        assert_eq!(mybreak.id, "break-id-1");
        assert_eq!(mybreak.description, "Lunch Break - Manually clock in/out");
    }

    #[test]
    fn it_can_fetch_active_policy() {
        let (mut server, client) = setup();
        let _m = server.mock_active_policy();

        let policy = client.active_break_policy().unwrap();
        assert_eq!(policy.break_policy, "some-break-policy-id");
        assert_eq!(policy.time_policy, "some-policy-id");
        assert_eq!(policy.role_overrides.role_properties.default_timezone, "Europe/Berlin");
    }
}
