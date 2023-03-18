use url::Url;

use crate::persistence;

#[derive(Clone, Debug)]
pub struct Session {
    access_token: String,
    pub company: Option<String>,
    pub role: Option<String>,
}

impl Session {
    pub fn new(token: String) -> Self {
        Self {
            access_token: token,
            company: None,
            role: None,
        }
    }

    pub fn load() -> Self {
        let state = persistence::State::load();
        Self {
            access_token: state.access_token.expect("State missing access token"),
            company: state.company_id,
            role: state.role_id,
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

    // pub fn company_and_role_set(&self) -> bool {
    //     [&self.company, &self.role].iter().all(|f| f.is_some())
    // }

    pub fn set_company_and_role(&mut self, company: String, role: String) {
        self.company = Some(company);
        self.role = Some(role);
    }

    pub fn company(&self) -> Option<&str> {
        self.company.as_ref().map(|s| s.as_str())
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_ref().map(|s| s.as_str())
    }

    pub fn get(&self, path: &str) -> attohttpc::RequestBuilder {
        self.request(attohttpc::Method::GET, path)
    }

    pub fn post(&self, path: &str) -> attohttpc::RequestBuilder {
        self.request(attohttpc::Method::POST, path)
    }

    pub fn request(&self, method: attohttpc::Method, path: &str) -> attohttpc::RequestBuilder {
        let mut builder = attohttpc::RequestBuilder::new(method, self.url_for(path)).bearer_auth(&self.access_token);
        if let Some(value) = &self.company {
            builder = builder.header("company", value);
        }
        if let Some(value) = &self.role {
            builder = builder.header("role", value);
        }
        builder
    }

    fn url_for(&self, path: &str) -> Url {
        #[cfg(not(test))]
        let url = "https://app.rippling.com/api/";
        #[cfg(test)]
        let url = &utilities::mocking::server_url();
        Url::parse(url).unwrap().join(path).unwrap()
    }
}
