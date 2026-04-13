use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list::MagicList;
use async_trait::async_trait;

#[async_trait]
pub trait MagicListRepository: Send + Sync {
    async fn create(&self, username: &String, command: CreateMagicListCommand) -> Result<(), Box<dyn ApplicationError>>;
    async fn find_by_id(&self, magic_list_id: i32) -> Result<MagicList, Box<dyn ApplicationError>>;
    async fn is_family_member(&self, username: &str, family_id: i32) -> Result<bool, Box<dyn ApplicationError>>;
    async fn add_item(&self, magic_list_id: i32, command: CreateMagicListItemCommand) -> Result<(), Box<dyn ApplicationError>>;
}
