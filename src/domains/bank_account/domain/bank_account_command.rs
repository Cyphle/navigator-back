use chrono::DateTime;
use crate::domains::common::big_decimal::BigDecimal;
use crate::domains::common::periodicity::Periodicity;
use crate::domains::common::visibility::Visibility;

pub struct CreateBankAccountCommand {
    pub name: String,
    pub description: String,
    pub visibility: Visibility,
    pub starting_amount: BigDecimal,
    pub start_date: DateTime<chrono::Utc>,
}

pub struct AddBudgetToAccountCommand {
    pub account_id: i32,
    pub name: String,
    pub description: String,
    pub start_date: DateTime<chrono::Utc>,
    pub end_date: Option<DateTime<chrono::Utc>>,
    pub initial_amount: BigDecimal,
}

pub struct AddExpenseToAccountCommand {
    pub account_id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub expense_date: DateTime<chrono::Utc>,
    pub debit_date: DateTime<chrono::Utc>,
}

pub struct AddExpenseToBudgetCommand {
    pub account_id: i32,
    pub budget_id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub expense_date: DateTime<chrono::Utc>,
    pub debit_date: DateTime<chrono::Utc>,
}

pub struct AddChargeToAccountCommand {
    pub account_id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub charge_date: DateTime<chrono::Utc>,
    pub debit_date: DateTime<chrono::Utc>,
    pub periodicity: Option<Periodicity>,
}

pub struct AddCreditToAccountCommand {
    pub account_id: i32,
    pub description: String,
    pub amount: BigDecimal,
    pub credit_date: DateTime<chrono::Utc>,
    pub debit_date: DateTime<chrono::Utc>,
}
