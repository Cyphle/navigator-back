use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::visibility::Visibility;
use crate::domains::family::domain::family::Family;
use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
use crate::testing::repositories::mock_bank_account_read_repository::MockBankAccountReadRepository;
use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
use crate::testing::repositories::mock_magic_list_repository::MockMagicListRepository;
use crate::testing::repositories::mock_user_repository::MockUserRepository;
use crate::testing::repositories::mock_database::MockPoolPostgres;
use crate::testing::security::oidc::dummy_oidc_config;
use actix_web::web;
use std::sync::Arc;

pub type MockActixState = ActixState<MockPoolPostgres>;

pub struct MockMagicListConfig {
    pub owner_username: String,
    pub visibility: Visibility,
    pub family_id: Option<i32>,
    pub is_family_member: bool,
    pub summaries: Vec<MagicListSummary>,
}

impl Default for MockMagicListConfig {
    fn default() -> Self {
        Self {
            owner_username: "mock_user".to_string(),
            visibility: Visibility::Shared,
            family_id: None,
            is_family_member: true,
            summaries: vec![],
        }
    }
}

#[derive(Default)]
pub struct MockStateConfig {
    pub families: Option<Vec<Family>>,
    pub user_should_error: bool,
    pub family_should_error: bool,
    pub magic_list: MockMagicListConfig,
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
        magic_list_repository: Arc::new(MockMagicListRepository {
            owner_username: config.magic_list.owner_username,
            visibility: config.magic_list.visibility,
            family_id: config.magic_list.family_id,
            is_family_member: config.magic_list.is_family_member,
            summaries: config.magic_list.summaries,
        }),
    })
}
