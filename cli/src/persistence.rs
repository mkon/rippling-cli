use std::sync::OnceLock;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

const APP_NAME: &str = "rippling-cli";
static STATE: OnceLock<State> = OnceLock::new();

pub fn state() -> &'static State {
    STATE.get_or_init(|| State::load())
}

fn load<T: Serialize + DeserializeOwned + Default>(name: &str) -> T {
    confy::load(APP_NAME, name).expect(&format!("Could not read {name}"))
}

fn store<T: Serialize>(name: &str, cfg: T) {
    confy::store(APP_NAME, name, cfg).expect(&format!("Could not write {name}"))
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub access_token: Option<String>,
}

impl ::std::default::Default for Settings {
    fn default() -> Self {
        Self { access_token: None }
    }
}

impl Settings {
    const CONFIG_NAME: &'static str = "config";

    pub fn load() -> Self {
        load::<Self>(Self::CONFIG_NAME)
    }

    pub fn store(&self) {
        store(Self::CONFIG_NAME, self)
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct State {
    pub company_id: Option<String>,
    pub role_id: Option<String>,
    pub token: Option<String>,
}

impl State {
    const CONFIG_NAME: &'static str = "state";

    #[allow(dead_code)]
    pub fn load() -> Self {
        load::<Self>(Self::CONFIG_NAME)
    }

    pub fn store(&self) {
        store(Self::CONFIG_NAME, self)
    }
}

impl Into<rippling_api::client::Client> for &State {
    fn into(self) -> rippling_api::client::Client {
        let client = rippling_api::client::Client::new(self.token.clone().unwrap());
        if let Some(company) = self.company_id.clone() {
            if let Some(role) = self.role_id.clone() {
                return client.with_company_and_role(company, role);
            }
        }
        client
    }
}
