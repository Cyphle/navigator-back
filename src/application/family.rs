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
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{
        mock_actix_state, MockActixState, MockStateConfig,
    };
    use crate::config::actix::ActixState;
    use crate::testing::repositories::mock_database::{MockPoolPostgres, MockPoolPostgresError};
    use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
    use crate::testing::repositories::mock_user_repository::MockUserRepository;
    use actix_web::web;

    fn make_state_ok() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![
                    FamilyEntity {
                        id: 1,
                        name: "Family A".to_string(),
                    },
                    FamilyEntity {
                        id: 2,
                        name: "Family B".to_string(),
                    },
                ]),
                ..MockStateConfig::default()
            },
        )
    }

    fn make_state_db_error() -> web::Data<ActixState<MockPoolPostgresError, MockUserRepository, MockFamilyRepository>> {
        mock_actix_state(MockPoolPostgresError, MockStateConfig::default())
    }

    fn make_state_repo_error() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                family_should_error: true,
                ..MockStateConfig::default()
            },
        )
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
