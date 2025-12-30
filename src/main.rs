mod config;
mod domain;
mod http;
mod repositories;
mod security;
mod testing;
mod application;

use crate::config::actix::ActixState;
use crate::config::database::connect;
use crate::http::controllers::technical::{live, ready};
use crate::http::controllers::user::users_me;
use crate::repositories::user::SqlxUserRepository;
use crate::security::controllers::login::login;
use crate::security::controllers::logout::logout;
use crate::security::controllers::register::register;
use crate::security::oidc::get_client;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use log::info;
use std::sync::{Arc, Mutex};
use crate::config::cors::actix_cors_config;
use crate::config::session::actix_session_config;
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
                            .wrap(actix_cors_config(&cors_config))
                            .wrap(actix_session_config(&config, session_key.clone(), session_store.clone()))
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
