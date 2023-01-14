use serde::{Deserialize, Serialize};

const APP_NAME: &str = "rippling-cli";
const CONFIG_NAME: &str = "config";

#[derive(Serialize, Deserialize)]
pub struct MyConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub company: Option<String>,
    pub employee: Option<String>,
    pub username: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl MyConfig {
    pub fn load() -> Self {
        confy::load(APP_NAME, CONFIG_NAME).unwrap()
    }

    pub fn store(&self) {
        confy::store(APP_NAME, CONFIG_NAME, self).expect("Could not write configuration")
    }
}

impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            client_id: None,
            client_secret: None,
            company: None,
            employee: None,
            username: None,
            access_token: None,
            refresh_token: None,
        }
    }
}