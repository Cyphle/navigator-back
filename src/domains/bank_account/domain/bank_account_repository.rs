use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use async_trait::async_trait;
use crate::domains::bank_account::domain::bank_account::BankAccount;
use crate::domains::bank_account::domain::bank_account_command::{AddChargeToAccountCommand, AddCreditToAccountCommand, AddExpenseToAccountCommand, AddExpenseToBudgetCommand, CreateBankAccountCommand};
use crate::domains::bank_account::domain::budget::Budget;

#[async_trait]
pub trait BankAccountRepository: Send + Sync {
    async fn get_bank_accounts_for(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        filter: &BankAccountFilter,
    ) -> Result<Vec<BankAccount>, sqlx::Error>;

    async fn create_bank_account(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        bank_account: CreateBankAccountCommand
    ) -> Result<BankAccount, sqlx::Error>;

    async fn add_budget_to_account(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        bank_account_id: i32,
        budget: Budget
    ) -> Result<(), sqlx::Error>;

    async fn add_expense_to_account(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        bank_account_id: i32,
        expense: AddExpenseToAccountCommand
    ) -> Result<(), sqlx::Error>;

    async fn add_charge_to_account(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        bank_account_id: i32,
        charge: AddChargeToAccountCommand
    ) -> Result<(), sqlx::Error>;

    async fn add_credit_to_account(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        bank_account_id: i32,
        credit: AddCreditToAccountCommand
    ) -> Result<(), sqlx::Error>;

    async fn add_budget_expense_to_account(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        bank_account_id: i32,
        budget_id: i32,
        expense: AddExpenseToBudgetCommand
    ) -> Result<(), sqlx::Error>;
}
