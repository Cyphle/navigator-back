use crate::config::actix::ActixState;
use crate::domains::bank_account::http::bank_account_views::BankAccountSummaryView;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::debug;

#[get("/families/{family_id}/bank-accounts/summary")]
pub async fn get_bank_account_summary_endpoint(
    _session: Session,
    _state: web::Data<ActixState>,
    family_id: web::Path<String>,
) -> impl Responder {
    debug!("[Controller] Get bank account summary of family {}", family_id);
    let summary = Vec::<BankAccountSummaryView>::new();
    HttpResponse::Ok().json(summary)
}