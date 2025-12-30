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
    use crate::config::actix::{ActixState, DbConnection};
    use crate::domain::user::user::User;
    use crate::repositories::family::FamilyEntity;
    use crate::repositories::user::{UserEntity, UserRepository};
    use crate::testing::repositories::mock_database::MockTransaction;
    use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
    use crate::testing::repositories::mock_user_repository::MockUserRepository;
    use crate::testing::security::oidc::dummy_oidc_config;
    use actix_web::web;
    use async_trait::async_trait;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    struct MockDbOk;

    impl DbConnection for MockDbOk {
        type Tx<'a> = MockTransaction;

        fn begin<'a>(
            &'a self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
            Box::pin(async { Ok(MockTransaction) })
        }
    }

    struct MockDbErr;

    impl DbConnection for MockDbErr {
        type Tx<'a> = MockTransaction;

        fn begin<'a>(
            &'a self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
            Box::pin(async { Err(sqlx::Error::RowNotFound) })
        }
    }

    struct MockUserRepositoryError;

    #[async_trait]
    impl UserRepository<MockTransaction> for MockUserRepositoryError {
        async fn create_user(
            &self,
            _tx: &mut MockTransaction,
            _user: &User,
        ) -> Result<(u64, actix_web::http::StatusCode), sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn get_user(
            &self,
            _tx: &mut MockTransaction,
            _username: &str,
        ) -> Result<UserEntity, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn get_or_create_user(
            &self,
            _tx: &mut MockTransaction,
            _user: &User,
        ) -> Result<UserEntity, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }
    }

    fn make_state_ok() -> web::Data<ActixState<MockDbOk, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockDbOk,
            oidc_config: dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepository::default()),
            family_repository: Arc::new(MockFamilyRepository {
                families: vec![FamilyEntity {
                    id: 1,
                    name: "Family A".to_string(),
                }],
            }),
        })
    }

    fn make_state_db_error(
    ) -> web::Data<ActixState<MockDbErr, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockDbErr,
            oidc_config: dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepository::default()),
            family_repository: Arc::new(MockFamilyRepository { families: vec![] }),
        })
    }

    fn make_state_repo_error(
    ) -> web::Data<ActixState<MockDbOk, MockUserRepositoryError, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockDbOk,
            oidc_config: dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepositoryError),
            family_repository: Arc::new(MockFamilyRepository { families: vec![] }),
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
