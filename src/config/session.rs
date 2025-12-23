use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SessionConfig {
    pub database: SessionDatabaseConfig,
    pub store_addr: String,
    pub cookie_name: Option<String>,
    pub session_ttl_days: u64
}

#[derive(Debug, Deserialize, Clone)]
pub struct SessionDatabaseConfig {
    pub host: String,
    pub port: String,
    pub password: String,
    pub username: String,
    pub db: u8,
}