use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list::MagicList;
use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
use crate::domains::magic_list::domain::update_magic_list_item_command::UpdateMagicListItemCommand;
use async_trait::async_trait;

#[async_trait]
pub trait MagicListRepository: Send + Sync {
    async fn create(&self, username: &str, command: CreateMagicListCommand) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, magic_list_id: i32) -> Result<MagicList, RepositoryError>;
    async fn get_summary_for_user_and_family(&self, username: &str, family_id: i32) -> Result<Vec<MagicListSummary>, RepositoryError>;
    async fn add_item(&self, magic_list_id: i32, command: CreateMagicListItemCommand) -> Result<(), RepositoryError>;
    async fn update_item(&self, magic_list_id: i32, item_id: i32, command: UpdateMagicListItemCommand) -> Result<(), RepositoryError>;
}
