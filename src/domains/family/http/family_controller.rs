use crate::config::actix::ActixState;
use crate::domains::family::http::family_middleware::{create_family_middleware, get_families_middleware};
use crate::domains::family::http::family_requests::CreateFamilyRequest;
use crate::domains::family::usecases::create_family_use_case::create_family_use_case;
use crate::domains::family::usecases::get_families_use_case::get_families_use_case;
use actix_session::Session;
use actix_web::{get, post, web, Responder};
use log::debug;

#[get("/families")]
pub async fn get_families_endpoint(session: Session, state: web::Data<ActixState>) -> impl Responder {
    debug!("[Controller] Get families");
    get_families_middleware(session, state, get_families_use_case).await
}

#[post("/families")]
pub async fn create_family_endpoint(
    payload: web::Json<CreateFamilyRequest>,
    session: Session,
    state: web::Data<ActixState>
) -> impl Responder {
    debug!("[Controller] Create family");
    create_family_middleware(session, state, payload.into_inner(), create_family_use_case).await
}
