use std::time::Duration;
use serde::Deserialize;
use sqlx::{Error, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub connect_timeout: u64,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64
}

pub async fn connect(config: &DatabaseConfig) -> Pool<Postgres> {
    let connexion_string = "postgres://".to_string() + &config.username + ":" + &config.password + "@" + &config.host + ":" + &config.port + "/" + &config.name;
    match PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .connect(&connexion_string)
        .await {
        Ok(pool) => pool,
        Err(error) => {
            log::error!("Failed to connect to database: {}", error);
            panic!("Failed to connect to database: {}", error);
        }
    }
}