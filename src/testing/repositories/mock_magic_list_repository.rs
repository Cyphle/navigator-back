use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list::MagicList;
use crate::domains::magic_list::domain::magic_list_repository::MagicListRepository;
use crate::domains::magic_list::domain::update_magic_list_item_command::UpdateMagicListItemCommand;
use async_trait::async_trait;

pub struct MockMagicListRepository {
    pub owner_username: String,
    pub visibility: Visibility,
    pub family_id: Option<i32>,
    pub is_family_member: bool,
}

#[async_trait]
impl MagicListRepository for MockMagicListRepository {
    async fn create(&self, _username: &String, _command: CreateMagicListCommand) -> Result<(), Box<dyn ApplicationError>> {
        Ok(())
    }

    async fn find_by_id(&self, magic_list_id: i32) -> Result<MagicList, Box<dyn ApplicationError>> {
        Ok(MagicList {
            id: magic_list_id,
            owner_username: self.owner_username.clone(),
            visibility: self.visibility.clone(),
            family_id: self.family_id,
        })
    }

    async fn is_family_member(&self, _username: &str, _family_id: i32) -> Result<bool, Box<dyn ApplicationError>> {
        Ok(self.is_family_member)
    }

    async fn add_item(&self, _magic_list_id: i32, _command: CreateMagicListItemCommand) -> Result<(), Box<dyn ApplicationError>> {
        Ok(())
    }

    async fn update_item(&self, _magic_list_id: i32, _item_id: i32, _command: UpdateMagicListItemCommand) -> Result<(), Box<dyn ApplicationError>> {
        Ok(())
    }
}
