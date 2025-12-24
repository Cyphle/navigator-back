use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;
use crate::config::actix::ActixState;
use crate::security::token::get_username_from_session;

#[derive(Serialize)]
struct UserView {
    username: String,
}

#[get("/users/me")]
pub async fn user_me(session: Session, state: web::Data<ActixState>) -> impl Responder {
    log::debug!("Calling users me");

    let client = state.oidc_client.as_ref().unwrap().lock().unwrap();
    match get_username_from_session(&client, &session).await {
        Some(username) => {
            debug!("Username in session: {:?}", username);

            let mut tx = match state.db_connection.begin().await {
                Ok(tx) => tx,
                Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
            };

            let result = state
                .user_repository
                .get_user(&mut tx, &username)
                .await;

            match result {
                Ok(user) => HttpResponse::Ok().json(UserView {
                    username: user.username
                }),
                Err(e) => {
                    error!("Error getting user profile: {:?}", e);
                    HttpResponse::Ok().finish()
                }
            }
        }
        None => {
            error!("No username info found in session");
            HttpResponse::Ok().finish()
        }
    }
}