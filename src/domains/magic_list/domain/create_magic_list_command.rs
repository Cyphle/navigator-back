use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::magic_list_type::MagicListType;

pub struct CreateMagicListCommand {
    pub name: String,
    pub visibility: Visibility,
    pub magic_list_type: MagicListType,
    pub family_id: Option<i32>,
    pub excluded_member_ids: Option<Vec<i32>>,
}
