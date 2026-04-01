use crate::domains::common::big_decimal::BigDecimal;
use crate::domains::common::periodicity::Periodicity;
use crate::domains::common::visibility::Visibility;
use chrono::DateTime;

pub struct BankAccount {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub visibility: Visibility,
    pub starting_amount: BigDecimal,
    pub start_date: DateTime<chrono::Utc>,
    pub budgets: Vec<Budget>,
    pub charges: Vec<Charge>,
    pub credits: Vec<Credit>,
    pub expenses: Vec<Expense>,
}

pub struct Budget {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub start_date: DateTime<chrono::Utc>,
    pub end_date: Option<DateTime<chrono::Utc>>,
    pub initial_amount: BigDecimal,
    pub expenses: Vec<Expense>,
}

pub struct Expense {
    pub id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub expense_date: DateTime<chrono::Utc>,
    pub debit_date: DateTime<chrono::Utc>,
}

pub struct Charge {
    pub id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub date: DateTime<chrono::Utc>,
    pub periodicity: Periodicity,
}

pub struct Credit {
    pub id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub date: DateTime<chrono::Utc>,
}

pub enum TransactionType {
    Expense,
    Charge,
    Credit,
    Budget,
}

impl TryFrom<&str> for TransactionType {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "EXPENSE" => Ok(TransactionType::Expense),
            "CHARGE" => Ok(TransactionType::Charge),
            "CREDIT" => Ok(TransactionType::Credit),
            "BUDGET" => Ok(TransactionType::Budget),
            _ => Err(()),
        }
    }
}