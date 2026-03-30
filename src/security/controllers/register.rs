use crate::config::actix::ActixState;
use crate::security::controllers::auth_request::AuthRequest;
use crate::security::token::get_admin_access_token;
use actix_session::Session;
use actix_web::web::Data;
use actix_web::{post, web, HttpResponse, Responder};
use log::{error, info};
use reqwest::Client as HttpClient;
use serde::Serialize;
use crate::domains::user::domain::create_user_command::CreateUserCommand;
use crate::domains::user::domain::user::User;
use crate::domains::user::domain::user_repository::UserRepository;

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
    pub password: String,
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
    let command = CreateUserCommand {
        username: request_payload.username.to_owned(),
        email: request_payload.email.to_owned(),
        first_name: request_payload.first_name.to_owned(),
        last_name: request_payload.last_name.to_owned(),
        password: request_payload.password.to_owned(),
    };

    let mut tx = match state.db_connection.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            let error_msg: String = e.to_string();
            return HttpResponse::InternalServerError().body(error_msg);
        }
    };

    let result: Result<User, sqlx::Error> = state
        .user_repository
        .create_user(&mut tx, &command)
        .await;

    match result {
        Ok(user) => {
            match state.oidc_client.as_ref() {
                Some(client) => {
                    if let Err(e) = tx.commit().await {
                        let error_msg: String = e.to_string();
                        return HttpResponse::InternalServerError().body(error_msg);
                    }

                    let client = client.lock().unwrap();
                    let admin_token = get_admin_access_token(&client, &state.oidc_config.admin).await.unwrap();

                    let new_user = KeycloakUser {
                        username: user.username.clone(),
                        email: user.email.clone(),
                        enabled: true,
                        credentials: vec![KeycloakCredential {
                            r#type: "password".to_string(),
                            value: command.password.clone(),
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
