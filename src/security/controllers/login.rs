use actix_session::Session;
use actix_web::web::Data;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::Duration;
use log::{error, info};
use openid::{Options, Token};
use crate::config::actix::ActixState;
use crate::config::application::USER_SESSION_KEY;
use crate::security::controllers::auth_request::AuthRequest;

#[get("/login")]
async fn login(
    session: Session,
    state: Data<ActixState>,
    query: web::Query<AuthRequest>,
) -> impl Responder {
    log::debug!("Received login request");

    let client = state.oidc_client.as_ref().unwrap().lock().unwrap();

    match &query.code {
        Some(authorization_code) => {
            info!(
                "Requesting token with received authorization code: {}",
                authorization_code
            );
    
            let nonce: Option<&str> = state.oidc_config.nonce.as_deref();
            let max_age_duration = state.oidc_config.get_max_age();
            let max_age: Option<&Duration> = max_age_duration.as_ref();
    
            match client.authenticate(authorization_code, nonce, max_age).await {
                Ok(token) => {
                    save_in_session(session, &token);
    
                    // TODO ici il faut rediriger vers le referer qui a lancé le login
                    HttpResponse::PermanentRedirect()
                        .append_header(("Location", "http://localhost:5173/"))
                        .finish()
                }
                Err(err) => {
                    error!("Error exchanging code for token: {:?}", err);
                    HttpResponse::InternalServerError().body("Failed to authenticate")
                }
            }
        }
        None => {
            info!("No code provided. Starting authentication.");
    
            let auth_url = client.auth_url(&Options {
                scope: Some("openid email profile".into()),
                nonce: state.oidc_config.nonce.clone(),
                max_age: state.oidc_config.get_max_age().clone(),
                ..Default::default()
            });
    
            HttpResponse::Found()
                .append_header(("Location", auth_url.to_string()))
                .finish()
        }
    }
}

fn save_in_session(session: Session, token: &Token) {
    let saved = session.insert(
        USER_SESSION_KEY,
        token.bearer.clone(),
    );
    match saved {
        Ok(_) => info!("Token saved in session"),
        Err(e) => error!("Failed to save token in session: {}", e)
    }
}
