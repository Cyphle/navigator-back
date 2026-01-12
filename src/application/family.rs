use crate::application::errors::ApplicationErrors;
use crate::config::actix::{ActixState, DbConnection, DbTransaction};
use crate::repositories::family::{FamilyEntity, FamilyRepository};
use crate::repositories::user::UserRepository;
use actix_web::web;
use serde::Serialize;
use std::error::Error;
use std::fmt;
use log::{error, info};
use crate::domain::family::family::{CreateFamilyCommand, Family, FamilyRole};

impl fmt::Display for ApplicationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationErrors::FamilyAlreadyExists => write!(f, "Family already exists"),
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

pub async fn get_families<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: String,
) -> Result<Vec<FamilyEntity>, ApplicationErrors>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(ApplicationErrors::Database)?;

    state
        .family_repository
        .get_families_for(&mut tx, &username)
        .await
        .map_err(ApplicationErrors::Database)
}

pub async fn create_family<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: String,
    command: CreateFamilyCommand,
) -> Result<Family, ApplicationErrors>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    info!("Creating family {} for user '{}'", &command.name, &username);

    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(ApplicationErrors::Database)?;

    match state.family_repository
        .get_family_by_name(&mut tx, username.as_str(), command.name.as_str())
        .await
    {
        Ok(_) => {
            error!("Family {} already exists for : {}", &command.name, &username);
            tx.rollback()
                .await
                .map_err(ApplicationErrors::Database)?;
            Err(ApplicationErrors::FamilyAlreadyExists)
        }
        Err(sqlx::Error::RowNotFound) => {
            let family_name = command.name.clone();
            if let Err(err) = state
                .family_repository
                .create_family(&mut tx, &username, command)
                .await
            {
                tx.rollback()
                    .await
                    .map_err(ApplicationErrors::Database)?;
                return Err(ApplicationErrors::Database(err));
            }

            tx.commit()
                .await
                .map_err(ApplicationErrors::Database)?;

            Ok(Family { name: family_name })
        }
        Err(err) => {
            tx.rollback()
                .await
                .map_err(ApplicationErrors::Database)?;
            Err(ApplicationErrors::Database(err))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApplicationErrors, CreateFamilyCommand, FamilyRole, create_family, get_families};
    use crate::config::actix::ActixState;
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{MockActixState, MockStateConfig, mock_actix_state};
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

    fn make_state_db_error()
    -> web::Data<ActixState<MockPoolPostgresError, MockUserRepository, MockFamilyRepository>> {
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
    async fn should_error_on_db_connection_failure() {
        let state = make_state_db_error();
        let result = get_families(state, "john".to_string()).await;

        assert!(matches!(
            result,
            Err(ApplicationErrors::Database(sqlx::Error::RowNotFound))
        ));
    }

    #[actix_web::test]
    async fn should_error_on_repository_failure() {
        let state = make_state_repo_error();
        let result = get_families(state, "john".to_string()).await;

        assert!(matches!(
            result,
            Err(ApplicationErrors::Database(sqlx::Error::RowNotFound))
        ));
    }

    #[actix_web::test]
    async fn should_return_families() {
        let state = make_state_ok();
        let result = get_families(state, "john".to_string()).await;

        let families = result.expect("families");
        assert_eq!(families.len(), 2);
        assert_eq!(families[0].name, "Family A");
        assert_eq!(families[1].name, "Family B");
    }

    #[actix_web::test]
    async fn should_create_family_when_not_exists() {
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![]),
                ..MockStateConfig::default()
            },
        );

        let result = create_family(
            state,
            "john".to_string(),
            CreateFamilyCommand {
                name: "Family C".to_string(),
                role: FamilyRole::Owner,
            },
        )
        .await;

        let family = result.expect("family");
        assert_eq!(family.name, "Family C");
    }

    #[actix_web::test]
    async fn should_error_when_family_already_exists() {
        let state = make_state_ok();
        let result = create_family(
            state,
            "john".to_string(),
            CreateFamilyCommand {
                name: "Family A".to_string(),
                role: FamilyRole::Owner,
            },
        )
        .await;

        assert!(matches!(result, Err(ApplicationErrors::FamilyAlreadyExists)));
    }

    #[actix_web::test]
    async fn should_error_when_create_family_fails() {
        let state = make_state_repo_error();
        let result = create_family(
            state,
            "john".to_string(),
            CreateFamilyCommand {
                name: "Family C".to_string(),
                role: FamilyRole::Owner,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(ApplicationErrors::Database(sqlx::Error::RowNotFound))
        ));
    }
}
