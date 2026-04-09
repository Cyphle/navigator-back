use crate::domains::bank_account::domain::bank_account::BankAccount;
use crate::domains::bank_account::domain::budget::Budget;
use crate::domains::bank_account::domain::charge::Charge;
use crate::domains::bank_account::domain::credit::Credit;
use crate::domains::bank_account::domain::expense::Expense;
use crate::domains::common::big_decimal::BigDecimal;
use actix_web::cookie::time::Month;
use chrono::{DateTime, TimeZone, Utc};

pub fn utc_date(year: i32, month: Month, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month as u32, day, 0, 0, 0).unwrap()
}

pub fn a_bank_account(
    starting_amount: BigDecimal,
    start_date: DateTime<chrono::Utc>,
    budgets: Vec<Budget>,
    charges: Vec<Charge>,
    credits: Vec<Credit>,
    expenses: Vec<Expense>,
) -> BankAccount {
    BankAccount {
        id: 0,
        name: "".to_string(),
        description: "".to_string(),
        visibility: crate::domains::common::visibility::Visibility::Personal,
        starting_amount,
        start_date,
        budgets,
        charges,
        credits,
        expenses,
    }
}

pub fn a_budget(
    start_date: DateTime<chrono::Utc>,
    initial_amount: BigDecimal,
    expenses: Vec<Expense>,
) -> Budget {
    Budget {
        id: 0,
        name: "".to_string(),
        description: "".to_string(),
        initial_amount,
        start_date,
        end_date: None,
        expenses
    }
}

pub fn an_expense(
    amount: BigDecimal,
    expense_date: DateTime<chrono::Utc>,
    debit_date: DateTime<chrono::Utc>,
) -> Expense {
    Expense {
        id: 0,
        description: "Je suis une dépenses".to_string(),
        amount,
        expense_date,
        debit_date,
    }
}

pub fn a_credit(
    amount: BigDecimal,
    date: DateTime<chrono::Utc>,
) -> Credit {
    Credit {
        id: 0,
        description: "Je suis un crédit".to_string(),
        amount,
        date,
    }
}

pub fn a_charge(
    amount: BigDecimal,
    date: DateTime<chrono::Utc>,
    periodicity: crate::domains::common::periodicity::Periodicity,
) -> Charge {
    Charge {
        id: 0,
        description: "Je suis une charge".to_string(),
        amount,
        date,
        periodicity,
    }
}