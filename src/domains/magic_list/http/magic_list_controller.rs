use crate::config::actix::ActixState;
use crate::domains::magic_list::http::magic_list_views::MagicListSummaryView;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
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