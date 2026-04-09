use chrono::DateTime;
use crate::domains::bank_account::domain::expense::Expense;
use crate::domains::common::big_decimal::BigDecimal;

#[derive(Debug, PartialEq, Clone)]
pub struct Budget {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub start_date: DateTime<chrono::Utc>,
    pub end_date: Option<DateTime<chrono::Utc>>,
    pub initial_amount: BigDecimal,
    pub expenses: Vec<Expense>,
}