use std::time::Duration;
use serde::Deserialize;
use sqlx::{ConnectOptions, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    #[allow(dead_code)]
    pub connect_timeout: u64,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub sqlx_logging: bool
}

pub async fn connect(config: &DatabaseConfig) -> Pool<Postgres> {
    use sqlx::postgres::PgConnectOptions;
    use std::str::FromStr;
    use log::LevelFilter;

    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.name
    );

    let options = PgConnectOptions::from_str(&connection_string)
        .expect("Invalid connection string")
        .log_statements(if config.sqlx_logging { LevelFilter::Debug } else { LevelFilter::Off })
        .log_slow_statements(LevelFilter::Warn, Duration::from_secs(1));

    match PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .connect_with(options)
        .await {
        Ok(pool) => pool,
        Err(error) => {
            log::error!("Failed to connect to database: {}", error);
            panic!("Failed to connect to database: {}", error);
        }
    }
}
