use actix_session::Session;
use actix_web::{get, web, Responder};
use log::debug;
use crate::config::actix::ActixState;
use crate::domains::dashboard::http::dashboard_middleware::get_dashboard_middleware;
use crate::domains::dashboard::usecases::get_dashboard_use_case::get_dashboard_use_case;

#[get("/dashboard/{family_id}")]
pub async fn get_dashboard_endpoint(
    session: Session,
    state: web::Data<ActixState>,
    family_id: web::Path<String>
) -> impl Responder {
    debug!("[Controller] Get dashboard of family {}", family_id);
    get_dashboard_middleware(session, state, family_id.into_inner(), get_dashboard_use_case).await
}
