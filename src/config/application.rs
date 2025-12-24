use crate::{config::database::DatabaseConfig, security::oidc::OidcConfig};
use crate::config::session::SessionConfig;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;
use log::info;
use crate::config::cors::CorsConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub app: AppServerConfig,
    pub database: DatabaseConfig,
    pub oidc: OidcConfig,
    pub session: SessionConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        info!("Loading application configuration...");
        let config_dir = std::env::var("NAVIGATOR_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"));
        let default_path = config_dir.join("default");
        let local_path = config_dir.join("local");

        let config = Config::builder()
            .add_source(File::with_name(default_path.to_string_lossy().as_ref()))
            .add_source(File::with_name(local_path.to_string_lossy().as_ref()).required(false))
            .add_source(Environment::with_prefix("NAVIGATOR").separator("_"))
            .build()?;

        config.try_deserialize()
    }

    pub fn get_session_store_url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}",
            self.session.database.username,
            self.session.database.password,
            self.session.database.host,
            self.session.database.port
        )
    }

    pub fn get_cookie_name(&self) -> String {
        self.session.cookie_name.as_ref().unwrap_or(&USER_SESSION_KEY.to_string()).clone()
    }
}

pub static USER_SESSION_KEY: &str = "navigator_user_session";
