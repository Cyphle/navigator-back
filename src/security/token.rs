use crate::config::actix::{ActixState, DbConnection};
use crate::config::application::USER_SESSION_KEY;
use crate::security::oidc::OidcAdminConfig;
use actix_session::Session;
use actix_web::web;
use log::{debug, error, warn};
use openid::{
    Bearer, Client, Discovered, DiscoveredClient, StandardClaims, Token,
    TokenIntrospection,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;

// To get the username_or_email from a Bearer token
pub async fn get_username_from_bearer(
    client: &DiscoveredClient,
    bearer: &Bearer,
) -> Option<String> {
    debug!("Bearer token found in session");
    let token_wrapper: Token<StandardClaims> = Token::from(bearer.clone());
    match client
        .request_token_introspection::<TokenIntrospection<StandardClaims>>(&token_wrapper)
        .await
    {
        Ok(userinfo) => userinfo.username,
        Err(e) => {
            warn!("No user info found: {}", e);
            None
        }
    }
}

// To get the username_or_email from the bearer token in session
pub async fn get_username_from_session(
    client: &DiscoveredClient,
    session: &Session,
) -> Option<String> {
    let bearer = session.get::<Bearer>(USER_SESSION_KEY);

    match bearer {
        Ok(Some(bearer)) => get_username_from_bearer(client, &bearer).await,
        Ok(None) => {
            error!("No bearer token found in session");
            None
        }
        Err(e) => {
            error!("Error getting bearer token from session: {}", e);
            None
        }
    }
}

// To get the connect username_or_email from session
pub async fn get_connected_username<DB>(session: &Session, state: &web::Data<ActixState<DB>>) -> Option<String>
where
    DB: DbConnection,
{
    #[cfg(test)]
    if let Ok(Some(username)) = session.get::<String>("test_username") {
        return Some(username);
    }

    let client = state.oidc_client.clone()?;
    let client = client.lock().unwrap();
    get_username_from_session(&client, &session).await
}

// Structure for the token response
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: u64,
}

// To get an access token for admin account (client)
pub async fn get_admin_access_token(
    client: &Client<Discovered, StandardClaims>,
    admin: &OidcAdminConfig
) -> Result<String, Box<dyn std::error::Error>> {
    let token_endpoint = client.config().token_endpoint.clone();
    let token_request = HttpClient::new()
        .post(token_endpoint)
        .basic_auth(admin.client.id.clone(), Some(admin.client.secret.clone()))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await;

    let response = token_request.map_err(|e| {
        error!("Error getting access token: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    response.json::<TokenResponse>().await
        .map(|r| r.access_token)
        .map_err(|e| {
            error!("Error getting access token: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })
}
