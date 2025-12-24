mod config;
mod domain;
mod repositories;
mod security;
mod http;

use crate::config::actix::ActixState;
use crate::config::database::connect;
use crate::repositories::user::SqlxUserRepository;
use crate::security::oidc::get_client;
use actix_cors::Cors;
use actix_session::SessionMiddleware;
use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::{Key, time};
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::{AnyPool, FromRow, Pool, Postgres};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::http::controllers::user::user_me;
use crate::security::controllers::login::login;
use crate::security::controllers::logout::logout;
use crate::security::controllers::register::register;

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

                    // Actix
                    let state = web::Data::new(ActixState {
                        oidc_config: config.oidc.clone(),
                        oidc_client: Some(oidc_client.clone()),

                        db_connection: connection,
                        user_repository: Arc::new(user_repository),
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
                                    .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
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

                            // Tests
                            .service(hello)
                            .service(echo)
                            .service(users)
                            .service(create_user)

                            // Login & stuffs
                            .service(login)
                            .service(logout)
                            .service(register)
                            .service(user_me)
                        // Technical
                        // .service(live)
                        // .service(ready)
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

// ===> EXAMPLES

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn get_user(pool: &Pool<Postgres>, id: i64) -> Result<(i32,), sqlx::Error> {
    // Postgres uses positional parameters like $1
    let row: (i32,) = sqlx::query_as("SELECT id FROM users LIMIT 1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

#[get("/users")]
async fn users() -> Result<impl Responder, actix_web::Error> {
    println!("Fetcing one user");
    sqlx::any::install_default_drivers();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5434/navigator")
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let row: (i32,) = get_user(&pool, 150_i64)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    assert_eq!(row.0, 150);

    // run migrations at startup
    // sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(HttpResponse::Ok().body("PostDB endpoint"))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[post("/users")]
async fn create_user(
    Json(payload): Json<CreateUser>,
) -> Result<(Json<UserDao>, StatusCode), actix_web::Error> {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5434/navigator")
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let (user, status) = create_user_with_pool(&pool, payload).await?;

    Ok((Json(user), status))
}

async fn create_user_with_pool(
    pool: &AnyPool,
    payload: CreateUser,
) -> Result<(UserDao, StatusCode), actix_web::Error> {
    // connect to the database
    // insert the user and get the generated id
    let row: (i32,) = sqlx::query_as("INSERT INTO users (username) VALUES ($1) RETURNING id")
        .bind(&payload.username)
        .fetch_one(pool)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let user = UserDao {
        id: row.0 as u64,
        username: payload.username,
    };

    Ok((user, StatusCode::CREATED))
}

// the input to our `create_user` handler
#[derive(Deserialize, Clone)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct UserDao {
    id: u64,
    username: String,
}
