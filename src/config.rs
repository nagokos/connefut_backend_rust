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
pub struct Config {
    pub database: Database,
}

impl Config {
    pub fn new() -> Result<Self> {
        let database = envy::prefixed("POSTGRES_").from_env::<Database>()?;

        let config = Config { database };
        Ok(config)
    }
}

pub fn get_config() -> &'static Config {
    &CONFIG
}
