use serde::Serialize;

#[derive(Serialize)]
pub struct BankAccountSummaryView {
    pub id: i32,
    pub name: String,
    pub visibility: String,
    pub actual_amount: f64,
    pub end_of_month_forecast: f64,
}

pub struct BudgetOverviewView {
    pub id: i32,
    pub name: String,
    pub initial_amount: f64,
    pub remaining_amount: f64,
}

pub struct BankAccountOverviewView {
    pub id: i32,
    pub name: String,
    pub visibility: String,
    pub starting_amount: f64,
    pub actual_amount: f64,
    pub remaining_amount: f64,
    pub total_credits: f64,
    pub total_expenses: f64,
    pub budgets: Vec<BudgetOverviewView>,
}