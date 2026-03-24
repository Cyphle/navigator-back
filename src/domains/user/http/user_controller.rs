use crate::config::actix::ActixState;
use crate::domains::user::http::user_middleware::users_info_middleware;
use actix_session::Session;
use actix_web::{get, web, Responder};
use log::debug;

#[get("/users/info")]
pub async fn users_info_endpoint(session: Session, state: web::Data<ActixState>) -> impl Responder {
    debug!("[Controller] Users me");
    users_info_middleware(session, state).await
}
