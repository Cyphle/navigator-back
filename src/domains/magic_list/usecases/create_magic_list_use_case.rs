use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::errors::CreateMagicListError;
use crate::domains::magic_list::domain::magic_list_type::MagicListType;
use actix_web::web;

pub async fn create_magic_list_use_case<DB: DbConnection + Clone + AsPgConn>(
    state: web::Data<ActixState<DB>>,
    username: String,
    name: String,
    visibility: String,
    magic_list_type: String,
    family_id: Option<i32>,
    excluded_member_ids: Option<Vec<i32>>,
) -> Result<(), CreateMagicListError> {
    let command = CreateMagicListCommand {
        name: name.clone(),
        visibility: parse_visibility(&visibility),
        magic_list_type: MagicListType::from_str(&magic_list_type),
        family_id,
        excluded_member_ids,
    };

    state
        .magic_list_repository
        .create(&username, command)
        .await
        .map_err(|source| CreateMagicListError::Repository { name, source })
}

fn parse_visibility(raw: &str) -> Visibility {
    match raw.to_uppercase().as_str() {
        "PERSONAL" => Visibility::Personal,
        _ => Visibility::Shared,
    }
}
