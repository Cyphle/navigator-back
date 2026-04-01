use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::bank_account::domain::bank_account::BankAccountOverview;
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::repository_error::RepositoryError;
use actix_web::web;

pub async fn get_bank_accounts_overviews_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    filter: BankAccountFilter,
) -> Result<Vec<BankAccountOverview>, Box<dyn ApplicationError>>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    let mut tx = state.db_connection.begin().await.map_err(|e| {
        Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>
    })?;

    state
        .bank_account_repository
        .get_bank_accounts_for(&mut tx, &username, &filter)
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)
}
