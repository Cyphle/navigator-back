use actix_web::{web, HttpResponse};
use log::{debug, error};
use crate::application::errors::ApplicationErrors;
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::{FamilyEntity, FamilyRepository};
use crate::repositories::user::{UserEntity, UserRepository};

pub async fn get_users_me<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: Option<String>,
) -> Result<UserEntity, ApplicationErrors>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    let username = username.ok_or(ApplicationErrors::MissingUsername)?;
    debug!("Username in session: {:?}", username);

    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(ApplicationErrors::Database)?;

    state
        .user_repository
        .get_user(&mut tx, &username)
        .await
        .map_err(ApplicationErrors::Database)
}