use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use actix_web::web;

#[derive(Debug)]
pub struct MagicListAccessDeniedError;

impl ApplicationError for MagicListAccessDeniedError {
    fn get_message(&self) -> String {
        "Access denied to this magic list".to_string()
    }
    fn status_code(&self) -> u16 { 403 }
}

pub async fn add_item_to_magic_list_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    magic_list_id: i32,
    command: CreateMagicListItemCommand,
) -> Result<(), Box<dyn ApplicationError>> {
    let magic_list = state.magic_list_repository.find_by_id(magic_list_id).await?;

    let authorized = if magic_list.owner_username == username {
        true
    } else if magic_list.visibility == Visibility::Shared {
        match magic_list.family_id {
            Some(family_id) => state.magic_list_repository.is_family_member(&username, family_id).await?,
            None => false,
        }
    } else {
        false
    };

    if !authorized {
        return Err(Box::new(MagicListAccessDeniedError));
    }

    state.magic_list_repository.add_item(magic_list_id, command).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
    use crate::domains::common::visibility::Visibility;
    use crate::testing::actix::mock_state::{mock_actix_state, MockMagicListConfig, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;

    fn a_command() -> CreateMagicListItemCommand {
        CreateMagicListItemCommand {
            title: "Buy milk".to_string(),
            content: None,
            checked: None,
            due_date: None,
            status: None,
        }
    }

    #[actix_web::test]
    async fn should_allow_owner_to_add_item() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Personal,
                ..Default::default()
            },
            ..Default::default()
        });
        let result = add_item_to_magic_list_use_case(state, "alice".to_string(), 1, a_command()).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_allow_family_member_on_shared_list() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Shared,
                family_id: Some(1),
                is_family_member: true,
            },
            ..Default::default()
        });
        let result = add_item_to_magic_list_use_case(state, "bob".to_string(), 1, a_command()).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_deny_non_owner_on_personal_list() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Personal,
                ..Default::default()
            },
            ..Default::default()
        });
        let result = add_item_to_magic_list_use_case(state, "bob".to_string(), 1, a_command()).await;
        assert_eq!(result.unwrap_err().get_message(), "Access denied to this magic list");
    }

    #[actix_web::test]
    async fn should_deny_non_family_member_on_shared_list() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Shared,
                family_id: Some(1),
                is_family_member: false,
            },
            ..Default::default()
        });
        let result = add_item_to_magic_list_use_case(state, "bob".to_string(), 1, a_command()).await;
        assert_eq!(result.unwrap_err().get_message(), "Access denied to this magic list");
    }
}
