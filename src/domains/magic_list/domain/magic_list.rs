use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::magic_list_type::MagicListType;

#[derive(Debug, Clone)]
pub struct MagicList {
    pub id: i32,
    pub name: String,
    pub list_type: MagicListType,
    pub owner_username: String,
    pub visibility: Visibility,
    pub family_id: Option<i32>,
}
