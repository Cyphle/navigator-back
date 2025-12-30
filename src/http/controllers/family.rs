use crate::config::actix::ActixState;
use crate::http::middlewares::family::get_families_middleware;
use actix_session::Session;
use actix_web::{get, web, Responder};

#[get("/families")]
pub async fn get_families_controller(session: Session, state: web::Data<ActixState>) -> impl Responder {
    get_families_middleware(session, state).await
}
