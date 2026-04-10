use crate::config::actix::{ActixState, DbConnection};
use crate::domains::family::domain::family::Family;
use crate::testing::repositories::mock_bank_account_read_repository::MockBankAccountReadRepository;
use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
use crate::testing::repositories::mock_magic_list_repository::MockMagicListRepository;
use crate::testing::repositories::mock_user_repository::MockUserRepository;
use crate::testing::repositories::mock_database::MockPoolPostgres;
use crate::testing::security::oidc::dummy_oidc_config;
use actix_web::web;
use std::sync::Arc;

pub type MockActixState = ActixState<MockPoolPostgres>;

#[derive(Default)]
pub struct MockStateConfig {
    pub families: Option<Vec<Family>>,
    pub user_should_error: bool,
    pub family_should_error: bool,
}

pub fn mock_actix_state<DB: DbConnection>(
    db_connection: DB,
    config: MockStateConfig,
) -> web::Data<ActixState<DB>> {
    web::Data::new(ActixState {
        db_connection,
        oidc_config: dummy_oidc_config(),
        oidc_client: None,
        user_repository: Arc::new(MockUserRepository {
            fixed_id: 1,
            fixed_username: "mock_user".to_string(),
            fixed_email: "mock_email".to_string(),
            fixed_first_name: "mock_first_name".to_string(),
            fixed_last_name: "mock_last_name".to_string(),
            should_error: config.user_should_error,
        }),
        family_repository: Arc::new(MockFamilyRepository {
            families: config.families.unwrap_or_default(),
            should_error: config.family_should_error,
        }),
        bank_account_repository: Arc::new(MockBankAccountReadRepository),
        magic_list_repository: Arc::new(MockMagicListRepository),
    })
}
