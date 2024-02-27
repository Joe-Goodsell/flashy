use std::path::PathBuf;

use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgConnectOptions;
use config::*;


#[derive(serde::Deserialize)]
pub struct Settings {
    //TODO: add app settings
    pub database: DatabaseSettings,
}


#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}


impl DatabaseSettings {
    pub fn get_connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .database(&self.database_name)
            .password(self.password.expose_secret())
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let root: PathBuf = std::env::current_dir().expect("Failed to identify current directory.");
    let config_dir: PathBuf = root.join("configuration");

    let config= Config::builder()
        .add_source(config::File::from(config_dir.join("config.yaml"))).build()?;

    //TODO: add telemetry
    config.try_deserialize::<Settings>()
} 