use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::update_magic_list_item_command::UpdateMagicListItemCommand;
use crate::domains::magic_list::usecases::check_magic_list_access::check_magic_list_access;
use actix_web::web;

pub async fn update_item_of_magic_list_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    magic_list_id: i32,
    item_id: i32,
    command: UpdateMagicListItemCommand,
) -> Result<(), Box<dyn ApplicationError>> {
    check_magic_list_access(&state, &username, magic_list_id).await?;
    state.magic_list_repository.update_item(magic_list_id, item_id, command).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::common::visibility::Visibility;
    use crate::testing::actix::mock_state::{mock_actix_state, MockMagicListConfig, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;

    fn a_command() -> UpdateMagicListItemCommand {
        UpdateMagicListItemCommand {
            title: Some("Updated title".to_string()),
            content: None,
            checked: None,
            due_date: None,
            status: None,
        }
    }

    #[actix_web::test]
    async fn should_allow_owner_to_update_item() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Personal,
                ..Default::default()
            },
            ..Default::default()
        });
        let result = update_item_of_magic_list_use_case(state, "alice".to_string(), 1, 10, a_command()).await;
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
                ..Default::default()
            },
            ..Default::default()
        });
        let result = update_item_of_magic_list_use_case(state, "bob".to_string(), 1, 10, a_command()).await;
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
        let result = update_item_of_magic_list_use_case(state, "bob".to_string(), 1, 10, a_command()).await;
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
                ..Default::default()
            },
            ..Default::default()
        });
        let result = update_item_of_magic_list_use_case(state, "bob".to_string(), 1, 10, a_command()).await;
        assert_eq!(result.unwrap_err().get_message(), "Access denied to this magic list");
    }
}
