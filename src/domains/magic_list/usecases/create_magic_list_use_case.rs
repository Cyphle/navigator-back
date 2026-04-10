use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use actix_web::web;

pub async fn create_magic_list_use_case<DB: DbConnection + Clone + AsPgConn>(
    state: web::Data<ActixState<DB>>,
    username: String,
    command: CreateMagicListCommand,
) -> Result<(), Box<dyn ApplicationError>> {
    state.magic_list_repository.create(&username, command).await
}
