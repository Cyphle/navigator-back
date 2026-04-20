use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::magic_list_type::MagicListType;

#[derive(Debug, Clone)]
pub struct MagicListSummary {
    pub id: i32,
    pub name: String,
    pub visibility: Visibility,
    pub magic_list_type: MagicListType,
    pub family_id: Option<i32>,
    pub item_count: i64,
}
