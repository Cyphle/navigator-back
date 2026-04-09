use chrono::DateTime;
use crate::domains::common::big_decimal::BigDecimal;

#[derive(Debug, PartialEq, Clone)]
pub struct Expense {
    pub id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub expense_date: DateTime<chrono::Utc>,
    pub debit_date: DateTime<chrono::Utc>,
}