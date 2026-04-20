use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::visibility::Visibility;
use actix_web::web;

#[derive(Debug)]
pub struct MagicListAccessDeniedError;

impl ApplicationError for MagicListAccessDeniedError {
    fn get_message(&self) -> String {
        "Access denied to this magic list".to_string()
    }
    fn status_code(&self) -> u16 { 403 }
}

pub async fn check_magic_list_access<DB: DbConnection>(
    state: &web::Data<ActixState<DB>>,
    username: &str,
    magic_list_id: i32,
) -> Result<(), Box<dyn ApplicationError>> {
    let magic_list = state.magic_list_repository.find_by_id(magic_list_id).await?;

    let authorized = if magic_list.owner_username == username {
        true
    } else if magic_list.visibility == Visibility::Shared {
        match magic_list.family_id {
            Some(family_id) => state.magic_list_repository.is_family_member(username, family_id).await?,
            None => false,
        }
    } else {
        false
    };

    if !authorized {
        return Err(Box::new(MagicListAccessDeniedError));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::common::visibility::Visibility;
    use crate::testing::actix::mock_state::{mock_actix_state, MockMagicListConfig, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;

    fn a_state(config: MockMagicListConfig) -> web::Data<ActixState<MockPoolPostgres>> {
        mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: config,
            ..Default::default()
        })
    }

    #[actix_web::test]
    async fn should_allow_owner_on_personal_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Personal,
            ..Default::default()
        });
        let result = check_magic_list_access(&state, "alice", 1).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_allow_owner_on_shared_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: Some(1),
            ..Default::default()
        });
        let result = check_magic_list_access(&state, "alice", 1).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_deny_non_owner_on_personal_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Personal,
            ..Default::default()
        });
        let result = check_magic_list_access(&state, "bob", 1).await;
        assert_eq!(result.unwrap_err().get_message(), "Access denied to this magic list");
    }

    #[actix_web::test]
    async fn should_allow_family_member_on_shared_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: Some(1),
            is_family_member: true,
            ..Default::default()
        });
        let result = check_magic_list_access(&state, "bob", 1).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_deny_non_family_member_on_shared_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: Some(1),
            is_family_member: false,
            ..Default::default()
        });
        let result = check_magic_list_access(&state, "bob", 1).await;
        assert_eq!(result.unwrap_err().get_message(), "Access denied to this magic list");
    }

    #[actix_web::test]
    async fn should_deny_non_owner_on_shared_list_without_family() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: None,
            ..Default::default()
        });
        let result = check_magic_list_access(&state, "bob", 1).await;
        assert_eq!(result.unwrap_err().get_message(), "Access denied to this magic list");
    }
}
