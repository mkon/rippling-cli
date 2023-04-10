use std::collections::HashMap;

use regex::Regex;
use serde::Deserialize;

use super::{session::Session, Error, Result};

#[derive(Debug)]
pub struct Client {
    id: String,
    secret: String,
}

impl Client {
    pub fn new(id: &str, secret: &str) -> Self {
        Self { id: id.to_owned(), secret: secret.to_owned() }
    }

    /// Returns a new public client with id and secret configured remotely
    pub fn initialize_from_remote() -> Result<Self> {
        let res = attohttpc::get(super::default_host().join("/login").unwrap()).send()?;
        let html = res.text()?;
        let re = Regex::new(r#"<script>window.ripplingConfig = (\{.*\})</script>"#).unwrap();
        match re.captures(&html) {
            Some(m) => {
                let data: HashMap<String, String> = serde_json::from_str(&m[1]).unwrap();
                let client = Self::new(data.get("CLIENT_ID").unwrap(), data.get("CLIENT_SECRET").unwrap());
                Ok(client)
            }
            None => Err(Error::UnexpectedPayload),
        }
    }

    /// Returns a new authenticated client
    pub fn authenticate(&self, username: &str, password: &str) -> Result<super::Session> {
        let params = [
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ];
        let req = attohttpc::post(super::default_root().join("o/token/").unwrap())
            .params(&params)
            .basic_auth(&self.id, Some(&self.secret));
        let result: TokenJson = req.send()?.json()?;
        Ok(Session::new(None, result.access_token))
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenJson {
    pub access_token: String,
}
