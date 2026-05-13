use crate::config::actix::{ActixState, AsPgConn, DbConnection, DbTransaction};
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::family::domain::create_family_command::{CreateFamilyCommand, CreateFamilyMemberCommand};
use crate::domains::family::domain::family::Family;
use crate::domains::family::domain::family_errors::CreateFamilyError;
use crate::domains::family::domain::family_relation::FamilyRelation;
use actix_web::web;
use log::info;

pub struct CreateFamilyMemberInput {
    pub username_or_email: String,
    pub relation: String,
    pub is_admin: bool,
}

pub async fn create_family_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    name: String,
    creator_relation: String,
    members: Vec<CreateFamilyMemberInput>,
) -> Result<Family, CreateFamilyError>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    info!("Creating family {} for user '{}'", &name, &username);

    let command = CreateFamilyCommand {
        name: name.clone(),
        creator_relation: FamilyRelation::from_str(&creator_relation),
        members: members
            .into_iter()
            .map(|m| CreateFamilyMemberCommand {
                username_or_email: m.username_or_email,
                relation: FamilyRelation::from_str(&m.relation),
                is_admin: m.is_admin,
            })
            .collect(),
    };

    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(|e| CreateFamilyError::Repository {
            name: name.clone(),
            source: RepositoryError::from(e),
        })?;

    let result = match state
        .family_repository
        .get_family_by_name(&mut tx, &username, &command.name)
        .await
    {
        Ok(_) => Err(CreateFamilyError::AlreadyExists { name: command.name.clone() }),
        Err(RepositoryError::NotFound) => state
            .family_repository
            .create_family(&mut tx, &username, &command)
            .await
            .map_err(|source| CreateFamilyError::Repository {
                name: command.name.clone(),
                source,
            }),
        Err(source) => Err(CreateFamilyError::Repository {
            name: command.name.clone(),
            source,
        }),
    };

    match &result {
        Ok(_) => tx.commit().await.map_err(|e| CreateFamilyError::Repository {
            name: name.clone(),
            source: RepositoryError::from(e),
        })?,
        Err(_) => tx.rollback().await.map_err(|e| CreateFamilyError::Repository {
            name: name.clone(),
            source: RepositoryError::from(e),
        })?,
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::actix::ActixState;
    use crate::domains::family::domain::family::Family;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::{MockPoolPostgres, MockPoolPostgresError};
    use actix_web::web;

    fn make_state_ok() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![
                    Family {
                        id: 1,
                        name: "Family A".to_string(),
                        creator_username: "johndoe".to_string(),
                        members: vec![],
                        active: true,
                    },
                    Family {
                        id: 2,
                        name: "Family B".to_string(),
                        creator_username: "johnsmith".to_string(),
                        members: vec![],
                        active: true,
                    },
                ]),
                ..MockStateConfig::default()
            },
        )
    }

    fn make_state_db_error() -> web::Data<ActixState<MockPoolPostgresError>> {
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
            "Family C".to_string(),
            "PARENT".to_string(),
            vec![],
        )
            .await;

        let family = result.expect("family");
        assert_eq!(family.name, "Family C");
    }

    #[actix_web::test]
    async fn should_error_when_family_already_exists() {
        let state = make_state_ok();
        let err = create_family_use_case(
            state,
            "johndoe".to_string(),
            "Family A".to_string(),
            "PARENT".to_string(),
            vec![],
        )
            .await
            .unwrap_err();

        assert!(matches!(err, CreateFamilyError::AlreadyExists { ref name } if name == "Family A"));
    }

    #[actix_web::test]
    async fn should_error_when_create_family_fails() {
        let state = make_state_repo_error();
        let result = create_family_use_case(
            state,
            "john".to_string(),
            "Family C".to_string(),
            "PARENT".to_string(),
            vec![],
        )
            .await;

        assert!(matches!(result, Err(CreateFamilyError::Repository { .. })));
    }

    #[actix_web::test]
    async fn should_error_on_db_connection_failure() {
        let state = make_state_db_error();
        let result = create_family_use_case(
            state,
            "john".to_string(),
            "Family C".to_string(),
            "PARENT".to_string(),
            vec![],
        ).await;

        assert!(matches!(result, Err(CreateFamilyError::Repository { .. })));
    }
}
