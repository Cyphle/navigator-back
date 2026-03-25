use serde::Serialize;

#[derive(Serialize)]
pub struct BankAccountSummaryView {
    pub id: i32,
    pub name: String,
    pub visibility: String,
    pub actual_amount: f64,
    pub end_of_month_forecast: f64,
}