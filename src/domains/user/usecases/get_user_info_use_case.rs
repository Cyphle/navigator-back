use actix_web::web;
use log::debug;
use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::user::domain::user::User;
use crate::domains::user::repositories::user_repository::UserRepository;

pub async fn get_user_info_use_case<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: Option<String>,
) -> Result<User, Box<dyn ApplicationError>>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    let username = username.ok_or_else(|| Box::new(MissingUsernameError) as Box<dyn ApplicationError>)?;
    debug!("Username in session: {:?}", username);

    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

    state
        .user_repository
        .get_user(&mut tx, &username)
        .await
        .map(|user| User {
            id: Some(user.id),
            username: user.username,
            email: user.email.unwrap_or("".to_string()),
            first_name: user.first_name,
            last_name: user.last_name,
        })
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)
}

#[cfg(test)]
mod tests {
    use super::get_user_info_use_case;
    use crate::testing::actix::mock_state::{
        mock_actix_state, MockActixState, MockStateConfig,
    };
    use crate::config::actix::ActixState;
    use crate::testing::repositories::mock_database::{MockPoolPostgres, MockPoolPostgresError};
    use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
    use crate::testing::repositories::mock_user_repository::MockUserRepository;
    use actix_web::web;
    use crate::domains::family::repositories::family_entity::FamilyEntity;

    fn make_state_ok() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![FamilyEntity {
                    id: 1,
                    name: "Family A".to_string(),
                }]),
                ..MockStateConfig::default()
            },
        )
    }

    fn make_state_db_error() -> web::Data<ActixState<MockPoolPostgresError, MockUserRepository, MockFamilyRepository>> {
        mock_actix_state(
            MockPoolPostgresError,
            MockStateConfig {
                families: Some(vec![]),
                ..MockStateConfig::default()
            },
        )
    }

    fn make_state_repo_error() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![]),
                user_should_error: true,
                ..MockStateConfig::default()
            },
        )
    }

    #[actix_web::test]
    async fn should_error_when_username_missing() {
        let state = make_state_ok();
        let result = get_user_info_use_case(state, None).await;
        assert!(result.is_err());
        let err = result.expect_err("should return error");
        assert_eq!(err.get_message(), "No username specified");
    }

    #[actix_web::test]
    async fn should_error_on_db_connection_failure() {
        let state = make_state_db_error();
        let result = get_user_info_use_case(state, Some("alice".to_string())).await;

        let err = result.expect_err("should return error");
        assert!(err.get_message().contains("no rows returned"));
    }

    #[actix_web::test]
    async fn should_error_on_repository_failure() {
        let state = make_state_repo_error();
        let result = get_user_info_use_case(state, Some("bob".to_string())).await;

        let err = result.expect_err("should return error");
        assert!(err.get_message().contains("no rows returned"));
    }

    #[actix_web::test]
    async fn should_return_user() {
        let state = make_state_ok();
        let result = get_user_info_use_case(state, Some("carol".to_string())).await;
        let user = result.expect("Expected user");
        assert_eq!(user.username, "mock_user");
    }
}
