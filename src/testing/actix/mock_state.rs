use std::sync::Arc;
use actix_web::web;
use crate::config::actix::ActixState;
use crate::repositories::family::FamilyEntity;
use crate::testing::repositories::mock_database::MockPoolPostgres;
use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
use crate::testing::repositories::mock_user_repository::MockUserRepository;
use crate::testing::security::oidc::dummy_oidc_config;

pub type MockActixState =
    ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>;

pub fn mock_actix_state(
    families: Vec<FamilyEntity>,
) -> web::Data<MockActixState> {
    web::Data::new(ActixState {
        db_connection: MockPoolPostgres,
        oidc_config: dummy_oidc_config(),
        oidc_client: None,
        user_repository: Arc::new(MockUserRepository::default()),
        family_repository: Arc::new(MockFamilyRepository {
            families,
        }),
    })
}
