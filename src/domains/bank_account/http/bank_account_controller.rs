use crate::config::actix::ActixState;
use crate::domains::bank_account::http::bank_account_views::BankAccountSummaryView;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::{debug, info};
use crate::domains::bank_account::http::bank_account_requests::RequestFilter;

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

#[get("/families/{family_id}/bank-accounts/overviews")]
pub async fn get_bank_accounts_overviews(
    _session: Session,
    _state: web::Data<ActixState>,
    family_id: web::Path<String>,
    filter: web::Query<RequestFilter>,
) -> impl Responder {
    info!("[Controller] Get bank accounts overviews of family {} and month {}", family_id, filter.date);
    HttpResponse::Ok().json(Vec::<BankAccountSummaryView>::new())
}