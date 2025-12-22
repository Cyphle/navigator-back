use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::StatusCode;
use actix_web::web::Json;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn get_user(pool: &Pool<Postgres>, id: i64) -> Result<(i64,), sqlx::Error> {
    // Postgres uses positional parameters like $1
    let row: (i64,) = sqlx::query_as("SELECT $1 FROM users")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

#[get("/users")]
async fn users() -> Result<impl Responder, actix_web::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/navigator")
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let row: (i64,) = get_user(&pool, 150_i64)
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


async fn create_user(
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), actix_web::Error> {
    // connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/navigator")
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // insert the user and get the generated id
    let row: (i64,) = sqlx::query_as("INSERT INTO users (username) VALUES ($1) RETURNING id")
        .bind(&payload.username)
        .fetch_one(&pool)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let user = User {
        id: row.0 as u64,
        username: payload.username,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(users)
            .service(create_user)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}