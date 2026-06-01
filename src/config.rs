use serde::{Deserialize, Serialize};

const APP: &str = "rdfwalk";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub bookmarks: Vec<String>,
}

pub fn load() -> Config {
    confy::load(APP, None).unwrap_or_default()
}

pub fn save(config: &Config) {
    let _ = confy::store(APP, None, config);
}
