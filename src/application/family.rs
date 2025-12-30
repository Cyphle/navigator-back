use std::error::Error;
use std::fmt;
use actix_web::web;
use crate::application::errors::ApplicationErrors;
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::{FamilyEntity, FamilyRepository};
use crate::repositories::user::UserRepository;

impl fmt::Display for ApplicationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationErrors::MissingUsername => write!(f, "No username provided"),
            ApplicationErrors::Database(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl Error for ApplicationErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ApplicationErrors::Database(err) => Some(err),
            _ => None,
        }
    }
}

pub async fn get_families_from_username<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: Option<String>,
) -> Result<Vec<FamilyEntity>, ApplicationErrors>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    let username = username.ok_or(ApplicationErrors::MissingUsername)?;

    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(ApplicationErrors::Database)?;

    state
        .family_repository
        .get_family_by_member_username(&mut tx, &username)
        .await
        .map_err(ApplicationErrors::Database)
}

#[cfg(test)]
mod tests {
    use super::{get_families_from_username, ApplicationErrors};
    use crate::config::actix::{ActixState, DbConnection};
    use crate::domain::user::user::User;
    use crate::repositories::family::{FamilyEntity, FamilyRepository};
    use crate::repositories::user::UserRepository;
    use actix_web::web;
    use async_trait::async_trait;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    struct MockTransaction;

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

    struct MockUserRepository;

    #[async_trait]
    impl UserRepository<MockTransaction> for MockUserRepository {
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
        ) -> Result<crate::repositories::user::UserEntity, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn get_or_create_user(
            &self,
            _tx: &mut MockTransaction,
            _user: &User,
        ) -> Result<crate::repositories::user::UserEntity, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }
    }

    struct MockFamilyRepository {
        families: Vec<FamilyEntity>,
        should_error: bool,
    }

    #[async_trait]
    impl FamilyRepository<MockTransaction> for MockFamilyRepository {
        async fn get_family_by_member_username(
            &self,
            _tx: &mut MockTransaction,
            _username: &str,
        ) -> Result<Vec<FamilyEntity>, sqlx::Error> {
            if self.should_error {
                Err(sqlx::Error::RowNotFound)
            } else {
                Ok(self.families.clone())
            }
        }
    }

    fn make_state_ok() -> web::Data<ActixState<MockDbOk, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockDbOk,
            oidc_config: crate::testing::security::oidc::dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepository),
            family_repository: Arc::new(MockFamilyRepository {
                families: vec![
                    FamilyEntity {
                        id: 1,
                        name: "Family A".to_string(),
                    },
                    FamilyEntity {
                        id: 2,
                        name: "Family B".to_string(),
                    },
                ],
                should_error: false,
            }),
        })
    }

    fn make_state_db_error() -> web::Data<ActixState<MockDbErr, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockDbErr,
            oidc_config: crate::testing::security::oidc::dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepository),
            family_repository: Arc::new(MockFamilyRepository {
                families: Vec::new(),
                should_error: false,
            }),
        })
    }

    fn make_state_repo_error() -> web::Data<ActixState<MockDbOk, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockDbOk,
            oidc_config: crate::testing::security::oidc::dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepository),
            family_repository: Arc::new(MockFamilyRepository {
                families: Vec::new(),
                should_error: true,
            }),
        })
    }

    #[actix_web::test]
    async fn should_error_when_username_missing() {
        let state = make_state_ok();
        let result = get_families_from_username(state, None).await;

        assert!(matches!(result, Err(ApplicationErrors::MissingUsername)));
    }

    #[actix_web::test]
    async fn should_error_on_db_connection_failure() {
        let state = make_state_db_error();
        let result = get_families_from_username(state, Some("john".to_string())).await;

        assert!(matches!(
            result,
            Err(ApplicationErrors::Database(sqlx::Error::RowNotFound))
        ));
    }

    #[actix_web::test]
    async fn should_error_on_repository_failure() {
        let state = make_state_repo_error();
        let result = get_families_from_username(state, Some("john".to_string())).await;

        assert!(matches!(
            result,
            Err(ApplicationErrors::Database(sqlx::Error::RowNotFound))
        ));
    }

    #[actix_web::test]
    async fn should_return_families() {
        let state = make_state_ok();
        let result = get_families_from_username(state, Some("john".to_string())).await;

        let families = result.expect("families");
        assert_eq!(families.len(), 2);
        assert_eq!(families[0].name, "Family A");
        assert_eq!(families[1].name, "Family B");
    }
}
