use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;

static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().expect("Unable to retrieve config"));

#[derive(Deserialize, Debug)]
pub struct Database {
    pub db: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Google {
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Line {
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub database: Database,
    pub google: Google,
    pub line: Line,
}

impl Config {
    pub fn new() -> Result<Self> {
        let database = envy::prefixed("POSTGRES_").from_env::<Database>()?;
        let google = envy::prefixed("GOOGLE_").from_env::<Google>()?;
        let line = envy::prefixed("LINE_").from_env::<Line>()?;

        let config = Config {
            database,
            google,
            line,
        };
        Ok(config)
    }
}

pub fn get_config() -> &'static Config {
    &CONFIG
}
