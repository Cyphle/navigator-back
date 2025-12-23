// use crate::config::actix::ActixState;
// use crate::security::token::get_admin_access_token;
// use actix_session::Session;
// use actix_web::web::Data;
// use actix_web::{get, post, web, HttpResponse, Responder};
// use log::{error, info};
// use openid::{Client, Discovered, StandardClaims};
// use reqwest::Client as HttpClient;
// use serde::{Deserialize, Serialize};
// use std::error::Error;
// use crate::repositories;
// use crate::security::controllers::auth_request::AuthRequest;
//
// #[derive(Serialize)]
// struct KeycloakUser {
//     username: String,
//     email: String,
//     enabled: bool,
//     credentials: Vec<KeycloakCredential>,
// }
//
// #[derive(Serialize)]
// struct KeycloakCredential {
//     r#type: String,
//     value: String,
//     temporary: bool,
// }
//
// #[derive(serde::Deserialize)]
// pub struct RegisterRequest {
//     pub username: String,
//     pub email: String,
//     pub first_name: String,
//     pub last_name: String,
//     pub password: String,
// }
//
// #[post("/register")]
// pub async fn register(
//     payload: web::Json<RegisterRequest>,
//     _: Session,
//     state: Data<ActixState>,
//     _: web::Query<AuthRequest>,
// ) -> impl Responder {
//     let request_payload = payload.into_inner();
//
//     match repositories::profile::create(
//         &state.db_connection,
//         &CreateUserCommand {
//             username: request_payload.username.to_owned(),
//             email: request_payload.email.to_owned(),
//             first_name: request_payload.first_name.to_owned(),
//             last_name: request_payload.last_name.to_owned(),
//         },
//     ).await {
//         Ok(profile) => {
//             match state.oidc_client.as_ref() {
//                 Some(client) => {
//                     let client = client.lock().unwrap();
//                     let admin_token = get_admin_access_token(&client, &state.oidc_config.admin).await.unwrap();
//
//                     let new_user = KeycloakUser {
//                         username: profile.username.clone(),
//                         email: profile.email.clone(),
//                         enabled: true,
//                         credentials: vec![KeycloakCredential {
//                             r#type: "password".to_string(),
//                             value: request_payload.password.to_owned().to_string(),
//                             temporary: false,
//                         }],
//                     };
//
//                     match HttpClient::new()
//                         .post(&state.oidc_config.admin.create_user_url)
//                         .bearer_auth(admin_token)
//                         .json(&new_user)
//                         .send()
//                         .await {
//                         Ok(response) => {
//                             info!("User created in keycloak: {:?}", response);
//                         }
//                         Err(e) => {
//                             println!("Error creating user in keycloak: {:?}", e);
//                         }
//                     }
//                 },
//                 None => {
//                     error!("OIDC client not found");
//                 },
//             }
//         },
//         Err(e) => {
//             error!("Error creating user: {:?}", e);
//         },
//     }
//
//     HttpResponse::Created().finish()
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::config::actix::ActixState;
//     use crate::security::controllers::register::register;
//     use actix_web::http::header::ContentType;
//     use actix_web::web;
//     use actix_web::{test, App};
//     use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
//     use sea_orm::prelude::DateTimeWithTimeZone;
//     use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};
//     use std::sync::Mutex;
//
//     fn get_mock_database() -> &'static DatabaseConnection {
//         let d = NaiveDate::from_ymd_opt(2015, 6, 3).unwrap();
//         let t = NaiveTime::from_hms_milli_opt(12, 34, 56, 789).unwrap();
//
//         Box::leak(Box::new(MockDatabase::new(DatabaseBackend::Postgres)
//             .append_query_results([
//                 vec![entity::profiles::Model {
//                     id: 1,
//                     username: "johndoe".to_owned(),
//                     email: "johndoe@banana.fr".to_owned(),
//                     first_name: "John".to_owned(),
//                     last_name: "Doe".to_owned(),
//                     created_at: DateTimeWithTimeZone::from_naive_utc_and_offset(NaiveDateTime::new(d, t), FixedOffset::east_opt(0).unwrap()),
//                     updated_at: DateTimeWithTimeZone::from_naive_utc_and_offset(NaiveDateTime::new(d, t), FixedOffset::east_opt(0).unwrap()),
//                     deleted_at: None,
//                 }],
//             ])
//             .into_connection()))
//     }
//
//     #[actix_web::test]
//     async fn should_register() {
//         let app = test::init_service(
//             App::new()
//                 .app_data(web::Data::new(AppState {
//                     db_connection: get_mock_database(),
//                     oidc_client: None,
//                     store: Mutex::new(std::collections::HashMap::new()),
//                     oidc_config: get_oidc_config().clone(),
//                 }))
//                 .service(register)
//         ).await;
//
//         let req = test::TestRequest::post()
//             .set_payload("{\"username\": \"johndoe\", \"email\": \"johndoe@banana.fr\", \"first_name\": \"John\", \"last_name\": \"Doe\", \"password\": \"Bonjour01\"}")
//             .insert_header(ContentType::json())
//             .uri("/register").to_request();
//
//         let resp = test::call_service(&app, req).await;
//         assert!(resp.status().is_success());
//     }
// }