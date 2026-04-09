use crate::domains::bank_account::domain::bank_account::BankAccount;
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

impl From<&BankAccount> for BankAccountOverviewView {
    fn from(bank_account: &BankAccount) -> Self {
        Self {
            id: bank_account.id,
            name: bank_account.name.clone(),
            visibility: bank_account.visibility.to_string(),
            starting_amount: bank_account.starting_amount.to_f64(),
            actual_amount: 0.0,
            remaining_amount: 0.0,
            total_credits: 0.0,
            total_expenses: 0.0,
            budgets: vec![],
        }
    }
}