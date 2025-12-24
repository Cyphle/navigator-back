use crate::config::actix::ActixState;
use crate::security::token::get_admin_access_token;
use actix_session::Session;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder};
use log::{error, info};
use openid::{Client, Discovered, StandardClaims};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::error::Error;
use crate::domain::user::user::User;
use crate::repositories;
use crate::security::controllers::auth_request::AuthRequest;

#[derive(Serialize)]
struct KeycloakUser {
    username: String,
    email: String,
    enabled: bool,
    credentials: Vec<KeycloakCredential>,
}

#[derive(Serialize)]
struct KeycloakCredential {
    r#type: String,
    value: String,
    temporary: bool,
}

#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: Option<String>,
}

#[post("/register")]
pub async fn register(
    payload: web::Json<RegisterRequest>,
    _: Session,
    state: Data<ActixState>,
    _: web::Query<AuthRequest>,
) -> impl Responder {
    log::debug!("Register reuqest");

    let request_payload = payload.into_inner();
    let user = User {
        username: request_payload.username.to_owned(),
    };

    let mut tx = match state.db_connection.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let result = state
        .user_repository
        .create_user(&mut tx, &user)
        .await;

    match result {
        Ok(_) => {
            match state.oidc_client.as_ref() {
                Some(client) => {
                    let client = client.lock().unwrap();
                    let admin_token = get_admin_access_token(&client, &state.oidc_config.admin).await.unwrap();

                    let new_user = KeycloakUser {
                        username: user.username.clone(),
                        email: user.username.clone(),
                        enabled: true,
                        credentials: vec![KeycloakCredential {
                            r#type: "password".to_string(),
                            value: request_payload.password.unwrap_or("coucou".to_string()),
                            temporary: false,
                        }],
                    };

                    match HttpClient::new()
                        .post(&state.oidc_config.admin.create_user_url)
                        .bearer_auth(admin_token)
                        .json(&new_user)
                        .send()
                        .await {
                        Ok(response) => {
                            info!("User created in keycloak: {:?}", response);
                        }
                        Err(e) => {
                            println!("Error creating user in keycloak: {:?}", e);
                        }
                    }
                },
                None => {
                    error!("OIDC client not found");
                },
            }
        },
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    HttpResponse::Created().finish()
}
