use crate::config::actix::ActixState;
use crate::domains::recipe::http::recipe_views::RecipeSummaryView;
use actix_session::Session;
use actix_web::{HttpResponse, Responder, get, web};
use log::debug;

#[get("/families/{family_id}/recipes/summary")]
pub async fn get_recipe_summary_endpoint(
    _session: Session,
    _state: web::Data<ActixState>,
    family_id: web::Path<String>,
) -> impl Responder {
    debug!("[Controller] Get recipe summary of family {}", family_id);
    let summary = Vec::<RecipeSummaryView>::new();
    HttpResponse::Ok().json(summary)
}
