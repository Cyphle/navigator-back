use crate::config::actix::ActixState;
use crate::domains::todo::http::todo_views::TodoSummaryView;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::debug;

#[get("/families/{family_id}/todos/summary")]
pub async fn get_todo_summary_endpoint(
    _session: Session,
    _state: web::Data<ActixState>,
    family_id: web::Path<String>
) -> impl Responder {
    debug!("[Controller] Get todo summary of family {}", family_id);
    let todo_summary = Vec::<TodoSummaryView>::new();
    HttpResponse::Ok().json(todo_summary)
}