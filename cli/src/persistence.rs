use std::sync::OnceLock;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

const APP_NAME: &str = "rippling-cli";
static STATE: OnceLock<State> = OnceLock::new();

pub fn state() -> &'static State {
    STATE.get_or_init(State::load)
}

fn load<T: Serialize + DeserializeOwned + Default>(name: &str) -> T {
    confy::load(APP_NAME, name).unwrap_or_else(|_| panic!("Could not read {name}"))
}

fn store<T: Serialize>(name: &str, cfg: T) {
    confy::store(APP_NAME, name, cfg).unwrap_or_else(|_| panic!("Could not write {name}"));
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct State {
    pub company_id: Option<String>,
    pub role_id: Option<String>,
    pub token: Option<String>,
}

impl State {
    const CONFIG_NAME: &'static str = "state";

    pub fn load() -> Self {
        load::<Self>(Self::CONFIG_NAME)
    }

    pub fn store(&self) {
        store(Self::CONFIG_NAME, self);
    }
}

impl From<&State> for rippling_api::client::Client {
    fn from(val: &State) -> Self {
        let client = rippling_api::client::Client::new(val.token.clone().unwrap());
        if let Some(company) = val.company_id.clone() {
            if let Some(role) = val.role_id.clone() {
                return client.with_company_and_role(company, role);
            }
        }
        client
    }
}
