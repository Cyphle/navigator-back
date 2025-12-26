mod config;
mod domain;
mod http;
mod repositories;
mod security;

use crate::config::actix::ActixState;
use crate::config::database::connect;
use crate::http::controllers::technical::{live, ready};
use crate::http::controllers::user::users_me;
use crate::repositories::user::SqlxUserRepository;
use crate::security::controllers::login::login;
use crate::security::controllers::logout::logout;
use crate::security::controllers::register::register;
use crate::security::oidc::get_client;
use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::{time, Key};
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::repositories::family::SqlxFamilyRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match config::application::AppConfig::new() {
        Ok(config) => {
            config::logger::config(&config.logging);

            // DATABASE
            info!("Connecting to database");
            let connection = connect(&config.database).await;

            info!("Connected to database");
            match sqlx::migrate!("./migrations").run(&connection).await {
                Ok(_) => {
                    info!("Database migrations completed successfully");

                    // OIDC
                    info!("Configuring OIDC client...");
                    let oidc_client = Arc::new(Mutex::new(get_client(&config.oidc).await));

                    // Session
                    info!("Configuring session store...");
                    let session_key = Key::from(&[0; 64]);
                    let session_store = RedisSessionStore::new(config.get_session_store_url())
                        .await
                        .unwrap();

                    // Clone config parts needed in the closure
                    let cors_config = config.cors.clone();
                    let app_config = config.app.clone();

                    // Repositories
                    let user_repository = SqlxUserRepository {};
                    let family_repository = SqlxFamilyRepository {};

                    // Actix
                    let state = web::Data::new(ActixState {
                        oidc_config: config.oidc.clone(),
                        oidc_client: Some(oidc_client.clone()),

                        db_connection: connection,
                        user_repository: Arc::new(user_repository),
                        family_repository: Arc::new(family_repository)
                    });

                    info!("Starting Actix server...");
                    HttpServer::new(move || {
                        App::new()
                            .wrap(
                                Cors::default()
                                    .allowed_origin(cors_config.allowed_origin.as_str())
                                    .allowed_methods(
                                        cors_config
                                            .allowed_methods
                                            .iter()
                                            .map(|m| m.parse::<actix_web::http::Method>().unwrap())
                                            .collect::<Vec<_>>(),
                                    )
                                    .allowed_headers(
                                        cors_config
                                            .allowed_headers
                                            .iter()
                                            .map(|h| {
                                                h.parse::<actix_web::http::header::HeaderName>()
                                                    .unwrap()
                                            })
                                            .collect::<Vec<_>>(),
                                    )
                                    .supports_credentials() // Optional, if credentials are used
                                    .max_age(3600),
                            )
                            .wrap(
                                SessionMiddleware::builder(
                                    session_store.clone(),
                                    session_key.clone(),
                                )
                                .session_lifecycle(
                                    PersistentSession::default()
                                        .session_ttl(time::Duration::days(5)),
                                )
                                .cookie_secure(false)
                                .cookie_name(config.get_cookie_name())
                                .build(),
                            )
                            .app_data(state.clone())
                            // Login & stuffs
                            .service(login)
                            .service(logout)
                            .service(register)
                            .service(users_me)
                            // Technical
                            .service(live)
                            .service(ready)
                    })
                    .bind(format!("{}:{}", app_config.host, app_config.port))?
                    .run()
                    .await
                }
                Err(e) => panic!("Failed to run database migrations: {:?}", e),
            }
        }
        Err(e) => {
            log::error!(
                "Error while loading application configuration: {:?}.\nCannot start server.",
                e
            );
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    }
}
