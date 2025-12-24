mod config;
mod security;
mod repositories;
mod domain;

use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use migration::{Migrator, MigratorTrait};
use crate::config::database::connect;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match config::application::AppConfig::new() {
        Ok(config) => {
            config::logger::config(&config.logging);

            // DATABASE
            info!("Connecting to database...");
            let db = connect(&config.database).await.unwrap();

            match Migrator::up(&db, None).await {
                Ok(_) => log::info!("Database migration completed successfully."),
                Err(e) => log::error!("Error while migrating database: {:?}.", e),
            }

            let static_db = Box::leak(Box::new(db));

            HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .service(echo)
            })
        }
        Err(e) => {
            // log::error!("Error while loading application configuration: {:?}.\nCannot start server.", e);
            println!("Error while loading application configuration: {:?}.\nCannot start server.", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Configuration error"));
        }
    }
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}



// ===> EXAMPLES

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
