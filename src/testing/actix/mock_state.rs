use std::sync::Arc;
use actix_web::web;
use crate::config::actix::ActixState;
use crate::repositories::family::FamilyEntity;
use crate::testing::repositories::mock_database::{MockPoolPostgres, MockPoolPostgresError};
use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
use crate::testing::repositories::mock_user_repository::MockUserRepository;
use crate::testing::security::oidc::dummy_oidc_config;

pub type MockActixState = ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>;
pub type MockActixStateDbError =
    ActixState<MockPoolPostgresError, MockUserRepository, MockFamilyRepository>;

#[derive(Default)]
pub struct MockStateConfig {
    pub families: Vec<FamilyEntity>,
    pub user_should_error: bool,
    pub family_should_error: bool,
}

pub fn mock_actix_state(families: Vec<FamilyEntity>) -> web::Data<MockActixState> {
    mock_actix_state_with(MockStateConfig {
        families,
        ..MockStateConfig::default()
    })
}

pub fn mock_actix_state_with(config: MockStateConfig) -> web::Data<MockActixState> {
    web::Data::new(ActixState {
        db_connection: MockPoolPostgres,
        oidc_config: dummy_oidc_config(),
        oidc_client: None,
        user_repository: Arc::new(MockUserRepository {
            fixed_id: 1,
            fixed_username: "mock_user".to_string(),
            should_error: config.user_should_error,
        }),
        family_repository: Arc::new(MockFamilyRepository {
            families: config.families,
            should_error: config.family_should_error,
        }),
    })
}

pub fn mock_actix_state_db_error(
    config: MockStateConfig,
) -> web::Data<MockActixStateDbError> {
    web::Data::new(ActixState {
        db_connection: MockPoolPostgresError,
        oidc_config: dummy_oidc_config(),
        oidc_client: None,
        user_repository: Arc::new(MockUserRepository {
            fixed_id: 1,
            fixed_username: "mock_user".to_string(),
            should_error: config.user_should_error,
        }),
        family_repository: Arc::new(MockFamilyRepository {
            families: config.families,
            should_error: config.family_should_error,
        }),
    })
}
