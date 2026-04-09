use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account::BankAccount;
use crate::domains::bank_account::domain::bank_account_command::{AddChargeToAccountCommand, AddCreditToAccountCommand, AddExpenseToAccountCommand, AddExpenseToBudgetCommand, CreateBankAccountCommand};
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::bank_account::domain::bank_account_repository::BankAccountRepository;
use async_trait::async_trait;
use crate::domains::bank_account::domain::budget::Budget;

pub struct MockBankAccountReadRepository;

#[async_trait]
impl BankAccountRepository for MockBankAccountReadRepository {
    async fn get_bank_accounts_for(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _filter: &BankAccountFilter,
    ) -> Result<Vec<BankAccount>, sqlx::Error> {
        Ok(vec![])
    }

    async fn create_bank_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _command: CreateBankAccountCommand,
    ) -> Result<BankAccount, sqlx::Error> {
        unimplemented!()
    }

    async fn add_budget_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _budget: Budget,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn add_expense_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _command: AddExpenseToAccountCommand,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn add_charge_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _command: AddChargeToAccountCommand,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn add_credit_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _command: AddCreditToAccountCommand,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn add_budget_expense_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _budget_id: i32,
        _command: AddExpenseToBudgetCommand,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }
}
