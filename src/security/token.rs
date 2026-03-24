use actix_session::Session;
use actix_web::web;
use crate::security::oidc::OidcAdminConfig;
use log::{error, info, warn};
use openid::{
    Bearer, Client, Discovered, DiscoveredClient, StandardClaims, Token,
    TokenIntrospection,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use crate::config::actix::{ActixState, DbConnection};
use crate::config::application::USER_SESSION_KEY;
use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::user::repositories::user_repository::UserRepository;

// To get the username from a Bearer token
pub async fn get_username_from_bearer(
    client: &DiscoveredClient,
    bearer: &Bearer,
) -> Option<String> {
    info!("Bearer token found in session {:?}", bearer.clone());
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

// To get the username from the bearer token in session
pub async fn get_username_from_session(
    client: &DiscoveredClient,
    session: &actix_session::Session,
) -> Option<String> {
    let bearer = session.get::<Bearer>(USER_SESSION_KEY);

    match bearer {
        Ok(bearer) => match bearer {
            Some(bearer) => get_username_from_bearer(client, &bearer).await,
            None => {
                error!("No bearer token found in session");
                None
            }
        },
        Err(e) => {
            error!("Error getting bearer token from session: {}", e);
            None
        }
    }
}

// To get the connect username from session
pub async fn get_connected_username<DB, U, F>(session: &Session, state: &web::Data<ActixState<DB, U, F>>) -> Option<String>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    #[cfg(test)]
    if let Ok(Some(username)) = session.get::<String>("test_username") {
        return Some(username);
    }

    let oidc_client = state.oidc_client.clone();
    match oidc_client {
        Some(client) => {
            let client = client.lock().unwrap();
            get_username_from_session(&client, &session).await
        }
        None => None,
    }
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

    match token_request {
        Ok(response) => {
            match response.json::<TokenResponse>().await {
                Ok(response) => Ok(response.access_token),
                Err(e) => {
                    error!("Error getting access token: {}", e);
                    Err(Box::new(e))
                }
            }
        }
        Err(e) => {
            error!("Error getting access token: {}", e);
            Err(Box::new(e))
        }
    }
}
