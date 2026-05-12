use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list_item_status::MagicListItemStatus;
use crate::domains::magic_list::usecases::check_magic_list_access::check_magic_list_access;
use actix_web::web;
use chrono::NaiveDate;

pub async fn add_item_to_magic_list_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    magic_list_id: i32,
    title: String,
    content: Option<String>,
    checked: Option<bool>,
    due_date: Option<NaiveDate>,
    status: Option<MagicListItemStatus>,
) -> Result<(), Box<dyn ApplicationError>> {
    check_magic_list_access(&state, &username, magic_list_id).await?;

    let command = CreateMagicListItemCommand {
        title,
        content,
        checked,
        due_date,
        status,
    };

    state.magic_list_repository.add_item(magic_list_id, command).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::common::visibility::Visibility;
    use crate::testing::actix::mock_state::{mock_actix_state, MockMagicListConfig, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;

    async fn call(state: web::Data<crate::testing::actix::mock_state::MockActixState>, user: &str) -> Result<(), Box<dyn ApplicationError>> {
        add_item_to_magic_list_use_case(
            state,
            user.to_string(),
            1,
            "Buy milk".to_string(),
            None,
            None,
            None,
            None,
        )
        .await
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
        assert!(call(state, "alice").await.is_ok());
    }

    #[actix_web::test]
    async fn should_allow_family_member_on_shared_list() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Shared,
                family_id: Some(1),
                ..Default::default()
            },
            is_family_member: true,
            ..Default::default()
        });
        assert!(call(state, "bob").await.is_ok());
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
        assert_eq!(
            call(state, "bob").await.unwrap_err().get_message(),
            "Access denied to this magic list"
        );
    }

    #[actix_web::test]
    async fn should_deny_non_family_member_on_shared_list() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                owner_username: "alice".to_string(),
                visibility: Visibility::Shared,
                family_id: Some(1),
                ..Default::default()
            },
            is_family_member: false,
            ..Default::default()
        });
        assert_eq!(
            call(state, "bob").await.unwrap_err().get_message(),
            "Access denied to this magic list"
        );
    }
}
