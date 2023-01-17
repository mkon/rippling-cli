use std::collections::HashMap;

use regex::Regex;
use serde::Deserialize;

use super::{Error, Result};

#[derive(Debug)]
pub struct Client {
    id: String,
    secret: String,
}

impl Client {
    pub fn new(id: &str, secret: &str) -> Self {
        Self {
            id: id.to_owned(),
            secret: secret.to_owned(),
        }
    }

    /// Returns a new public client with id and secret configured remotely
    pub fn initialize_from_remote() -> Result<Self> {
        let res = reqwest::blocking::get("https://app.rippling.com/login")?;
        let html = res.text()?;
        let re = Regex::new(r#"<script>window.ripplingConfig = (\{.*\})</script>"#).unwrap();
        match re.captures(&html) {
            Some(m) => {
                let data: HashMap<String, String> = serde_json::from_str(&m[1]).unwrap();
                let client = Self::new(data.get("CLIENT_ID").unwrap(), data.get("CLIENT_SECRET").unwrap());
                Ok(client)
            },
            None => Err(Error {  }),
        }
    }

    /// Returns a new authenticated client
    pub fn authenticate(&self, username: &str, password: &str) -> Result<super::AuthenticatedClient> {
        let params = [
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ];
        let req = reqwest::blocking::Client::new()
            .post("https://app.rippling.com/api/o/token/")
            .form(&params)
            .basic_auth(&self.id, Some(&self.secret));
        let result: TokenJson = req.send()?.json()?;

        Ok(super::AuthenticatedClient::new(&result.access_token))
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenJson {
    pub access_token: String,
}