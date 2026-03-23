use crate::config::actix::ActixState;
use crate::application::family::{create_family, get_families};
use crate::http::middlewares::family::{create_family_middleware, get_families_middleware};
use crate::http::requests::family::CreateFamilyRequest;
use crate::security::controllers::register::RegisterRequest;
use actix_session::Session;
use actix_web::{get, post, web, Responder};
use log::debug;

#[get("/families")]
pub async fn get_families_endpoint(session: Session, state: web::Data<ActixState>) -> impl Responder {
    debug!("[Controller] Get families");
    get_families_middleware(session, state, get_families).await
}

#[post("/families")]
pub async fn create_family_endpoint(
    payload: web::Json<CreateFamilyRequest>,
    session: Session,
    state: web::Data<ActixState>
) -> impl Responder {
    debug!("[Controller] Create family");
    create_family_middleware(session, state, payload.into_inner(), create_family).await
}
