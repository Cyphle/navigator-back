use chrono::Duration;
use openid::{Client, StandardClaims};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct OidcAdminConfig {
    pub client: OidcClientConfig,
    pub create_user_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OidcClientConfig {
    pub id: String,
    pub secret: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct OidcConfig {
    pub url: String,
    #[allow(dead_code)]
    pub realm_name: String,
    pub redirect_uri: String,
    pub logout_uri: String,
    pub client: OidcClientConfig,
    pub nonce: Option<String>,
    pub session_timeout_minutes: Option<i64>,
    pub admin: OidcAdminConfig,
}

impl OidcConfig {
    pub fn get_max_age(&self) -> Option<Duration> {
        self.session_timeout_minutes.map(|minutes| Duration::minutes(minutes))
    }
}

pub async fn get_client(config: &OidcConfig) -> Client<openid::Discovered, StandardClaims> {
    Client::discover(
        config.client.id.to_string(),
        config.client.secret.to_string(),
        Some(config.redirect_uri.to_string()),
        reqwest::Url::parse(&config.url).unwrap(),
    )
    .await
    .expect("Failed to discover OpenID configuration")
}
