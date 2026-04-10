use crate::config::actix::ActixState;
use crate::domains::magic_list::http::magic_list_middleware::create_magic_list_middleware;
use crate::domains::magic_list::http::magic_list_requests::CreateMagicListRequest;
use crate::domains::magic_list::http::magic_list_views::MagicListSummaryView;
use crate::domains::magic_list::usecases::create_magic_list_use_case::create_magic_list_use_case;
use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use log::debug;

#[get("/families/{family_id}/magic-lists/summary")]
pub async fn get_magic_list_summary_endpoint(
    _session: Session,
    _state: web::Data<ActixState>,
    family_id: web::Path<String>
) -> impl Responder {
    debug!("[Controller] Get magic_list summary of family {}", family_id);
    let magic_list_summary = Vec::<MagicListSummaryView>::new();
    HttpResponse::Ok().json(magic_list_summary)
}

#[post("/families/{family_id}/magic-lists")]
pub async fn create_magic_list_endpoint(
    payload: web::Json<CreateMagicListRequest>,
    session: Session,
    state: web::Data<ActixState>,
    family_id: web::Path<i32>
) -> impl Responder {
    debug!("[Controller] Create magic list for family {}", family_id);
    create_magic_list_middleware(
        session,
        state,
        family_id.into_inner(),
        payload.into_inner(),
        create_magic_list_use_case
    ).await
}