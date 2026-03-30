use crate::config::actix::ActixState;
use crate::security::controllers::auth_request::AuthRequest;
use crate::security::token::get_admin_access_token;
use actix_session::Session;
use actix_web::web::Data;
use actix_web::{post, web, HttpResponse, Responder};
use log::{error, info};
use reqwest::Client as HttpClient;
use serde::Serialize;
use crate::domains::user::domain::user::User;
use crate::domains::user::repositories::user_sqlx_repository::UserRepository;

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
#[allow(dead_code)]
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
        id: None,
        username: request_payload.username.to_owned(),
        email: request_payload.email.to_owned(),
        first_name: request_payload.first_name.to_owned(),
        last_name: request_payload.last_name.to_owned(),
    };

    let mut tx = match state.db_connection.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            let error_msg: String = e.to_string();
            return HttpResponse::InternalServerError().body(error_msg);
        }
    };

    let result: Result<(u64, actix_web::http::StatusCode), sqlx::Error> = state
        .user_repository
        .create_user(&mut tx, &user)
        .await;

    match result {
        Ok(_) => {
            match state.oidc_client.as_ref() {
                Some(client) => {
                    if let Err(e) = tx.commit().await {
                        let error_msg: String = e.to_string();
                        return HttpResponse::InternalServerError().body(error_msg);
                    }

                    let client = client.lock().unwrap();
                    let admin_token = get_admin_access_token(&client, &state.oidc_config.admin).await.unwrap();

                    // TODO là y a deux fois username_or_email
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
                            let status = response.status();
                            let body = response.text().await.unwrap_or_default();

                            if status.is_success() {
                                info!("User created in keycloak: status={}, body={}", status, body);
                            } else {
                                error!("Keycloak create user failed: status={}, body={}", status, body);
                            }
                        }
                        Err(e) => {
                            error!("Error creating user in keycloak: {:?}", e);
                        }
                    }
                },
                None => {
                    error!("OIDC client not found");
                },
            }
        },
        Err(e) => {
            let error_msg: String = e.to_string();
            return HttpResponse::InternalServerError().body(error_msg);
        }
    }

    HttpResponse::Created().finish()
}
