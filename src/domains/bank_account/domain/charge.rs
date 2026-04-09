use chrono::DateTime;
use crate::domains::common::big_decimal::BigDecimal;
use crate::domains::common::periodicity::Periodicity;

#[derive(Debug, PartialEq, Clone)]
pub struct Charge {
    pub id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub date: DateTime<chrono::Utc>,
    pub periodicity: Periodicity,
}