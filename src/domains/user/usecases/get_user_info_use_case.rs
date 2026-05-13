use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::user::domain::errors::GetUserInfoError;
use crate::domains::user::domain::user::User;
use actix_web::web;
use log::debug;

pub async fn get_user_info_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
) -> Result<User, GetUserInfoError>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    debug!("Username in session: {:?}", username);

    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(|e| GetUserInfoError::Repository {
            username: username.clone(),
            source: RepositoryError::from(e),
        })?;

    state
        .user_repository
        .get_user(&mut tx, &username)
        .await
        .map_err(|source| GetUserInfoError::Repository { username, source })
}
