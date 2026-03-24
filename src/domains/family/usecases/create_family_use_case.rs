use crate::config::actix::{ActixState, DbConnection, DbTransaction};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::Family;
use crate::domains::family::domain::family_errors::FamilyAlreadyExistsError;
use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::user::repositories::user_repository::UserRepository;
use actix_web::web;
use log::{error, info};

pub async fn create_family_use_case<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: String,
    command: CreateFamilyCommand,
) -> Result<Family, Box<dyn ApplicationError>>
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
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

    match state.family_repository
        .get_family_by_name(&mut tx, username.as_str(), command.name.as_str())
        .await
    {
        Ok(_) => {
            error!("Family {} already exists for : {}", &command.name, &username);
            tx.rollback()
                .await
                .map_err(|e: sqlx::Error| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
            Err(Box::new(FamilyAlreadyExistsError { name: command.name.clone() }))
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
                    .map_err(|e: sqlx::Error| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
                return Err(Box::new(RepositoryError { error: err.to_string() }));
            }

            tx.commit()
                .await
                .map_err(|e: sqlx::Error| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

            Ok(Family { name: family_name })
        }
        Err(err) => {
            tx.rollback()
                .await
                .map_err(|e: sqlx::Error| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
            Err(Box::new(RepositoryError { error: err.to_string() }))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::actix::ActixState;
    use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
    use crate::domains::family::domain::family_role::FamilyRole;
    use crate::domains::family::repositories::family_entity::FamilyEntity;
    use crate::domains::family::usecases::create_family_use_case::create_family_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
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
    async fn should_create_family_when_not_exists() {
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![]),
                ..MockStateConfig::default()
            },
        );

        let result = create_family_use_case(
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
        let result = create_family_use_case(
            state,
            "john".to_string(),
            CreateFamilyCommand {
                name: "Family A".to_string(),
                role: FamilyRole::Owner,
            },
        )
            .await;

        let err = result.expect_err("should return error");
        assert_eq!(err.get_message(), "Family already exists: Family A");
    }

    #[actix_web::test]
    async fn should_error_when_create_family_fails() {
        let state = make_state_repo_error();
        let result = create_family_use_case(
            state,
            "john".to_string(),
            CreateFamilyCommand {
                name: "Family C".to_string(),
                role: FamilyRole::Owner,
            },
        )
            .await;

        let err = result.expect_err("should return error");
        assert!(err.get_message().contains("no rows returned"));
    }
}
