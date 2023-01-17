use serde::{Deserialize, Serialize, de::DeserializeOwned};

const APP_NAME: &str = "rippling-cli";

fn load<T: Serialize + DeserializeOwned + Default>(name: &str) -> T {
    confy::load(APP_NAME, name).expect(&format!("Could not read {name}"))
}

fn store<T: Serialize>(name: &str, cfg: T) {
    confy::store(APP_NAME, name, cfg).expect(&format!("Could not write {name}"))
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub username: Option<String>,
}

impl ::std::default::Default for Settings {
    fn default() -> Self {
        Self {
            username: None,
        }
    }
}

impl Settings {
    const CONFIG_NAME: &str = "config";

    pub fn load() -> Self {
        load::<Self>(Self::CONFIG_NAME)
    }

    pub fn store(&self) {
        store(Self::CONFIG_NAME, self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub access_token: Option<String>,
    pub company_id: Option<String>,
    pub role_id: Option<String>,
}

impl ::std::default::Default for State {
    fn default() -> Self {
        Self {
            access_token: None,
            company_id: None,
            role_id: None,
        }
    }
}

impl State {
    const CONFIG_NAME: &str = "state";

    pub fn load() -> Self {
        load::<Self>(Self::CONFIG_NAME)
    }

    pub fn store(&self) {
        store(Self::CONFIG_NAME, self)
    }
}