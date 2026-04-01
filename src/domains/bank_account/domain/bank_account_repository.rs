use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account::BankAccount;
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use async_trait::async_trait;

#[async_trait]
pub trait BankAccountReadRepository: Send + Sync {
    async fn get_bank_accounts_for(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        filter: &BankAccountFilter,
    ) -> Result<Vec<BankAccount>, sqlx::Error>;
}
