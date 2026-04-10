use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use async_trait::async_trait;

#[async_trait]
pub trait MagicListRepository: Send + Sync {
    async fn create(&self, username: &String, command: CreateMagicListCommand) -> Result<(), Box<dyn ApplicationError>>;
}
