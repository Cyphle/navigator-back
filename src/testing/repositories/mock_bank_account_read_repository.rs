use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account::BankAccount;
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::bank_account::domain::bank_account_repository::BankAccountReadRepository;
use async_trait::async_trait;

pub struct MockBankAccountReadRepository;

#[async_trait]
impl BankAccountReadRepository for MockBankAccountReadRepository {
    async fn get_bank_accounts_for(
        &self,
        _conn: &mut dyn AsPgConn,
        _username: &str,
        _filter: &BankAccountFilter,
    ) -> Result<Vec<BankAccount>, sqlx::Error> {
        Ok(vec![])
    }
}
