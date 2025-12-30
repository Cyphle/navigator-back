use actix_web::web;
use log::debug;
use crate::application::errors::ApplicationErrors;
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::FamilyRepository;
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

#[cfg(test)]
mod tests {
    use super::get_users_me;
    use crate::application::errors::ApplicationErrors;
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{
        mock_actix_state_db_error, mock_actix_state_with, MockActixState,
        MockActixStateDbError, MockStateConfig,
    };
    use actix_web::web;

    fn make_state_ok() -> web::Data<MockActixState> {
        mock_actix_state_with(MockStateConfig {
            families: vec![FamilyEntity {
                id: 1,
                name: "Family A".to_string(),
            }],
            ..MockStateConfig::default()
        })
    }

    fn make_state_db_error() -> web::Data<MockActixStateDbError> {
        mock_actix_state_db_error(MockStateConfig {
            families: vec![],
            ..MockStateConfig::default()
        })
    }

    fn make_state_repo_error() -> web::Data<MockActixState> {
        mock_actix_state_with(MockStateConfig {
            families: vec![],
            user_should_error: true,
            ..MockStateConfig::default()
        })
    }

    #[actix_web::test]
    async fn should_error_when_username_missing() {
        let state = make_state_ok();
        let result = get_users_me(state, None).await;
        assert!(matches!(result, Err(ApplicationErrors::MissingUsername)));
    }

    #[actix_web::test]
    async fn should_error_on_db_connection_failure() {
        let state = make_state_db_error();
        let result = get_users_me(state, Some("alice".to_string())).await;
        assert!(matches!(result, Err(ApplicationErrors::Database(_))));
    }

    #[actix_web::test]
    async fn should_error_on_repository_failure() {
        let state = make_state_repo_error();
        let result = get_users_me(state, Some("bob".to_string())).await;
        assert!(matches!(result, Err(ApplicationErrors::Database(_))));
    }

    #[actix_web::test]
    async fn should_return_user() {
        let state = make_state_ok();
        let result = get_users_me(state, Some("carol".to_string())).await;
        let user = result.expect("Expected user");
        assert_eq!(user.username, "mock_user");
    }
}
