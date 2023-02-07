use serde::Deserialize;

use super::{Error, Result, Session};

pub fn fetch(session: &Session) -> Result<AccountInfo> {
    let req = session.get("auth_ext/get_account_info")?;
    super::request_to_result(req, |r| {
        let list = r.json::<Vec<AccountInfo>>()?;
        list.into_iter().next().ok_or(Error::UnexpectedPayload)
    })
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
pub struct TimeTrackPolicy {
    #[serde(rename = "breakPolicy")]
    pub break_policy_id: String,
}

#[cfg(test)]
mod tests {
    use utilities::mocking;
    use super::*;

    fn session() -> Session {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("some-company-id".into(), "some-role-id".into());
        session
    }

    #[test]
    fn it_can_fetch_account_info() {
        let _m = mocking::with_fixture("GET", "/auth_ext/get_account_info", "account_info").create();

        let info = fetch(&session()).unwrap();
        assert_eq!(info.role.company.id, "some-company-id");
        assert_eq!(info.id, "my-role-id");
    }
}
