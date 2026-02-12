use ureq::typestate::{WithBody, WithoutBody};

use crate::default_root;

#[derive(Debug)]
pub struct Client {
    company: Option<String>,
    role: Option<String>,
    root: url::Url,
    token: String,
}

/// Getters & instantiation
impl Client {
    pub fn new(token: String) -> Self {
        Self { company: None, role: None, root: default_root(), token }
    }

    pub fn role(&self) -> Option<&String> {
        self.role.as_ref()
    }

    pub fn company(&self) -> Option<&String> {
        self.company.as_ref()
    }

    pub fn with_company_and_role(&self, company: String, role: String) -> Self {
        Self {
            company: Some(company),
            role: Some(role),
            root: self.root.clone(),
            token: self.token.clone(),
        }
    }

    /// Used for testing mainly
    #[allow(dead_code)]
    pub(crate) fn with_root(&self, root: url::Url) -> Self {
        Self {
            company: self.company.clone(),
            role: self.role.clone(),
            root,
            token: self.token.clone(),
        }
    }
}

/// Methods for internal use
impl Client {
    fn agent(&self) -> ureq::Agent {
        ureq::agent()
    }

    pub(super) fn get(&self, path: &str) -> ureq::RequestBuilder<WithoutBody> {
        let mut request = self
            .agent()
            .get(self.root.join(path).unwrap().as_str())
            .header("Authorization", &format!("Bearer {}", self.token));
        if let Some(company) = &self.company {
            request = request.header("Company", company);
        }
        if let Some(role) = &self.role {
            request = request.header("Role", role);
        }
        request
    }

    pub(super) fn post(&self, path: &str) -> ureq::RequestBuilder<WithBody> {
        let mut request = self
            .agent()
            .post(self.root.join(path).unwrap().as_str())
            .header("Authorization", &format!("Bearer {}", self.token));
        if let Some(company) = &self.company {
            request = request.header("Company", company);
        }
        if let Some(role) = &self.role {
            request = request.header("Role", role);
        }
        request
    }
}
