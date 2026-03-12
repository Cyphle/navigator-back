use crate::config::actix::ActixState;
use crate::http::middlewares::user::users_info_middleware;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::debug;

#[get("/users/info")]
pub async fn users_info(session: Session, state: web::Data<ActixState>)  -> impl Responder {
    debug!("[Controller] Users me");
    users_info_middleware(session, state).await
}
