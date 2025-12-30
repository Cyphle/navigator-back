use actix_session::config::PersistentSession;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::{time, Key};
use serde::Deserialize;
use crate::config::application::AppConfig;

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

pub fn actix_session_config(config: &AppConfig, session_key: Key, session_store: RedisSessionStore) -> SessionMiddleware<RedisSessionStore> {
    SessionMiddleware::builder(
        session_store,
        session_key,
    )
        .session_lifecycle(
            PersistentSession::default()
                .session_ttl(time::Duration::days(config.session.session_ttl_days as i64)),
        )
        .cookie_secure(false)
        .cookie_name(config.get_cookie_name())
        .build()
}
