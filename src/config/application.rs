use crate::{config::database::DatabaseConfig, security::oidc::OidcConfig};
use crate::config::session::SessionConfig;
use config::{Config, Environment, File};
use serde::Deserialize;
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
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("NAVIGATOR").separator("_"))
            .build()?;

        config.try_deserialize()
    }

    pub fn get_session_store_url(&self) -> String {
        let base_url = if self.session.store_addr.is_empty() {
            format!(
                "redis://{}:{}@{}:{}",
                self.session.database.username,
                self.session.database.password,
                self.session.database.host,
                self.session.database.port
            )
        } else {
            self.session.store_addr.clone()
        };
        format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            self.session.database.db
        )
    }

    pub fn get_cookie_name(&self) -> String {
        self.session.cookie_name.as_ref().unwrap_or(&USER_SESSION_KEY.to_string()).clone()
    }
}

pub static USER_SESSION_KEY: &str = "navigator_user_session";
