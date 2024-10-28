use serde::Deserialize;

use super::{Error, Result};

impl super::Client {
    pub fn account_info(&self) -> Result<AccountInfo> {
        let list: Vec<AccountInfo> = self.get("auth_ext/get_account_info/").call()?.into_json()?;
        list.into_iter().next().ok_or(Error::UnexpectedPayload)
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

#[cfg(test)]
mod tests {
    use crate::Client;

    use utilities::mocking;

    fn setup() -> (mocking::FakeRippling, Client) {
        let server = mocking::FakeRippling::new();
        let client = Client::new("access-token".to_owned())
            .with_root(url::Url::parse(&server.url()).unwrap())
            .with_company_and_role("some-company-id".to_owned(), "some-role-id".to_owned());
        (server, client)
    }

    #[test]
    fn it_can_fetch_account_info() {
        let (mut server, client) = setup();
        let _m = server
            .with_fixture("GET", "/auth_ext/get_account_info/", "account_info")
            .create();

        let info = client.account_info().unwrap();
        assert_eq!(info.role.company.id, "some-company-id");
        assert_eq!(info.id, "my-role-id");
    }
}
