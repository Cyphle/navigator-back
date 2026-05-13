use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::family::domain::family::Family;
use crate::domains::family::domain::family_errors::GetFamiliesError;
use actix_web::web;

pub async fn get_families_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
) -> Result<Vec<Family>, GetFamiliesError>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    let mut tx = state
        .db_connection
        .begin()
        .await
        .map_err(|e| GetFamiliesError::Repository {
            username: username.clone(),
            source: RepositoryError::from(e),
        })?;

    state
        .family_repository
        .get_families_for(&mut tx, &username)
        .await
        .map_err(|source| GetFamiliesError::Repository { username, source })
}


#[cfg(test)]
mod tests {
    use super::get_families_use_case;
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
    async fn should_error_on_db_connection_failure() {
        let state = make_state_db_error();
        let result = get_families_use_case(state, "john".to_string()).await;
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn should_error_on_repository_failure() {
        let state = make_state_repo_error();
        let result = get_families_use_case(state, "john".to_string()).await;
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn should_return_families() {
        let state = make_state_ok();
        let result = get_families_use_case(state, "john".to_string()).await;

        let families = result.expect("families");
        assert_eq!(families.len(), 2);
        assert_eq!(families[0].name, "Family A");
        assert_eq!(families[1].name, "Family B");
    }
}
