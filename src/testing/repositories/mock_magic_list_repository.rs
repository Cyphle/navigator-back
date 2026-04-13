use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::magic_list_repository::MagicListRepository;
use async_trait::async_trait;

pub struct MockMagicListRepository;

#[async_trait]
impl MagicListRepository for MockMagicListRepository {
    async fn create(&self, _username: &String, _command: CreateMagicListCommand) -> Result<(), Box<dyn ApplicationError>> {
        Ok(())
    }
}
