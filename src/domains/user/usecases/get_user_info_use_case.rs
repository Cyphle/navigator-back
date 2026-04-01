use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::user::domain::user::User;
use actix_web::web;
use log::debug;

pub async fn get_user_info_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: Option<String>,
) -> Result<User, Box<dyn ApplicationError>>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    let username =
        username.ok_or_else(|| Box::new(MissingUsernameError) as Box<dyn ApplicationError>)?;
    debug!("Username in session: {:?}", username);

    let mut tx = state.db_connection.begin().await.map_err(|e| {
        Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>
    })?;

    state
        .user_repository
        .get_user(&mut tx, &username)
        .await
        .map_err(|e| {
            Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>
        })
}
