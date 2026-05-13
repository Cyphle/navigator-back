use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account::BankAccount;
use crate::domains::bank_account::domain::bank_account_command::{AddChargeToAccountCommand, AddCreditToAccountCommand, AddExpenseToAccountCommand, AddExpenseToBudgetCommand, CreateBankAccountCommand};
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::bank_account::domain::bank_account_repository::BankAccountRepository;
use crate::domains::common::errors::repository_error::RepositoryError;
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
    ) -> Result<Vec<BankAccount>, RepositoryError> {
        Ok(vec![])
    }

    async fn create_bank_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _command: CreateBankAccountCommand,
    ) -> Result<BankAccount, RepositoryError> {
        unimplemented!()
    }

    async fn add_budget_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _budget: Budget,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn add_expense_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _command: AddExpenseToAccountCommand,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn add_charge_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _command: AddChargeToAccountCommand,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn add_credit_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _command: AddCreditToAccountCommand,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn add_budget_expense_to_account(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _bank_account_id: i32,
        _budget_id: i32,
        _command: AddExpenseToBudgetCommand,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }
}
