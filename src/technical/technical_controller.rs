use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/health/live")]
pub async fn live() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "UP"
    }))
}

#[get("/health/ready")]
pub async fn ready() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "UP"
    }))
}