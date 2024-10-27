use serde::Deserialize;

use super::{Error, Result, Session};

pub fn fetch(session: &Session) -> Result<AccountInfo> {
    let list: Vec<AccountInfo> = session.get_json("auth_ext/get_account_info")?;
    list.into_iter().next().ok_or(Error::UnexpectedPayload)
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub id: String,
    pub role: AccountInfoRole,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfoRole {
    #[serde(rename = "_id")]
    pub id: Oid,
    pub company: Oid,
}

#[derive(Debug, Deserialize)]
pub struct Oid {
    #[serde(rename = "$oid")]
    pub id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTrackPolicy {
    #[serde(rename = "breakPolicy")]
    pub break_policy_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use utilities::mocking;

    #[test]
    fn it_can_fetch_account_info() {
        let mut server = mocking::FakeRippling::new();
        let _m = server
            .with_fixture("GET", "/auth_ext/get_account_info", "account_info")
            .create();

        let info = fetch(&crate::session::test_session(&server)).unwrap();
        assert_eq!(info.role.company.id, "some-company-id");
        assert_eq!(info.id, "my-role-id");
    }
}
