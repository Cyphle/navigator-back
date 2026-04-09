use crate::config::actix::{ActixState, AsPgConn, DbConnection, DbTransaction};
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::repository_error::RepositoryError;
use actix_web::web;
use crate::domains::bank_account::http::bank_account_views::BankAccountOverviewView;

pub async fn get_bank_accounts_overviews_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    filter: BankAccountFilter,
) -> Result<Vec<BankAccountOverviewView>, Box<dyn ApplicationError>>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

    match state
        .bank_account_repository
        .get_bank_accounts_for(&mut tx, &username, &filter)
        .await
    {
        Ok(bank_accounts) => {
            Ok(bank_accounts
                .iter()
                .map(|bank_account| BankAccountOverviewView::from(bank_account))
                .collect())
        }
        Err(err) => {
            tx.rollback()
                .await
                .map_err(|e: sqlx::Error| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
            Err(Box::new(RepositoryError { error: err.to_string() }))
        }
    }
}
