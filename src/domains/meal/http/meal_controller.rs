use crate::config::actix::ActixState;
use crate::domains::calendar::http::calendar_views::CalendarSummaryView;
use actix_session::Session;
use actix_web::{HttpResponse, Responder, get, web};
use log::debug;
use crate::domains::meal::http::meal_views::{empty, MealSummaryView};

#[get("/families/{family_id}/meals/summary")]
pub async fn get_meal_summary_endpoint(
    session: Session,
    state: web::Data<ActixState>,
    family_id: web::Path<String>,
) -> impl Responder {
    debug!("[Controller] Get calendar summary of family {}", family_id);
    let summary = empty();
    HttpResponse::Ok().json(summary)
}
