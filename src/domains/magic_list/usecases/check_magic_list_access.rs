use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::errors::CheckMagicListAccessError;
use actix_web::web;

pub async fn check_magic_list_access<DB: DbConnection>(
    state: &web::Data<ActixState<DB>>,
    username: &str,
    magic_list_id: i32,
) -> Result<(), CheckMagicListAccessError> {
    let magic_list = state
        .magic_list_repository
        .find_by_id(magic_list_id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => CheckMagicListAccessError::NotFound { magic_list_id },
            source => CheckMagicListAccessError::Repository { magic_list_id, source },
        })?;

    let authorized = if magic_list.owner_username == username {
        true
    } else if magic_list.visibility == Visibility::Shared {
        match magic_list.family_id {
            Some(family_id) => state
                .family_repository
                .is_family_member(username, family_id)
                .await
                .map_err(|source| CheckMagicListAccessError::Repository { magic_list_id, source })?,
            None => false,
        }
    } else {
        false
    };

    if !authorized {
        return Err(CheckMagicListAccessError::AccessDenied { magic_list_id });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::common::visibility::Visibility;
    use crate::testing::actix::mock_state::{mock_actix_state, MockMagicListConfig, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;

    fn a_state(magic_list: MockMagicListConfig, is_family_member: bool) -> web::Data<ActixState<MockPoolPostgres>> {
        mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list,
            is_family_member,
            ..Default::default()
        })
    }

    #[actix_web::test]
    async fn should_allow_owner_on_personal_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Personal,
            ..Default::default()
        }, false);
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
        }, false);
        let result = check_magic_list_access(&state, "alice", 1).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_deny_non_owner_on_personal_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Personal,
            ..Default::default()
        }, false);
        let err = check_magic_list_access(&state, "bob", 1).await.unwrap_err();
        assert!(matches!(err, CheckMagicListAccessError::AccessDenied { magic_list_id: 1 }));
    }

    #[actix_web::test]
    async fn should_allow_family_member_on_shared_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: Some(1),
            ..Default::default()
        }, true);
        let result = check_magic_list_access(&state, "bob", 1).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn should_deny_non_family_member_on_shared_list() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: Some(1),
            ..Default::default()
        }, false);
        let err = check_magic_list_access(&state, "bob", 1).await.unwrap_err();
        assert!(matches!(err, CheckMagicListAccessError::AccessDenied { magic_list_id: 1 }));
    }

    #[actix_web::test]
    async fn should_deny_non_owner_on_shared_list_without_family() {
        let state = a_state(MockMagicListConfig {
            owner_username: "alice".to_string(),
            visibility: Visibility::Shared,
            family_id: None,
            ..Default::default()
        }, false);
        let err = check_magic_list_access(&state, "bob", 1).await.unwrap_err();
        assert!(matches!(err, CheckMagicListAccessError::AccessDenied { magic_list_id: 1 }));
    }
}
