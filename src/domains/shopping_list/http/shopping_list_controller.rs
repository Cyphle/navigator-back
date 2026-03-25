use crate::config::actix::ActixState;
use crate::domains::shopping_list::http::shopping_list_views::ShoppingListSummaryView;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::debug;

#[get("/families/{family_id}/shopping-lists/summary")]
pub async fn get_shopping_list_summary_endpoint(
    _session: Session,
    _state: web::Data<ActixState>,
    family_id: web::Path<String>,
) -> impl Responder {
    debug!("[Controller] Get calendar summary of family {}", family_id);
    let summary = Vec::<ShoppingListSummaryView>::new();
    HttpResponse::Ok().json(summary)
}
