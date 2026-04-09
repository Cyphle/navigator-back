use chrono::DateTime;
use crate::domains::common::big_decimal::BigDecimal;

#[derive(Debug, PartialEq, Clone)]
pub struct Credit {
    pub id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub date: DateTime<chrono::Utc>,
}