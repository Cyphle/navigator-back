use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::debug;
use crate::config::actix::ActixState;
use crate::domains::calendar::http::calendar_views::CalendarSummaryView;

#[get("/families/{family_id}/calendars/summary")]
pub async fn get_calendar_summary_endpoint(
  session: Session,
  state: web::Data<ActixState>,
  family_id: web::Path<String>
) -> impl Responder {
    debug!("[Controller] Get calendar summary of family {}", family_id);
    let calendar_summary = Vec::<CalendarSummaryView>::new();
    HttpResponse::Ok().json(calendar_summary)
}