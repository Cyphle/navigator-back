use crate::config::actix::ActixState;
use crate::http::middlewares::user::users_me_middleware;
use actix_session::Session;
use actix_web::{get, web, Responder};
use log::debug;

#[get("/users/me")]
pub async fn users_me(session: Session, state: web::Data<ActixState>) -> impl Responder {
    debug!("[Controller] Users me");
    users_me_middleware(session, state).await
}
