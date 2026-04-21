use chrono::NaiveDate;

use crate::domains::magic_list::domain::magic_list_item_status::MagicListItemStatus;

#[derive(Debug, Clone)]
pub struct MagicListItem {
    pub id: i32,
    pub magic_list_id: i32,
    pub title: String,
    pub content: Option<String>,
    pub checked: bool,
    pub due_date: Option<NaiveDate>,
    pub status: Option<MagicListItemStatus>,
}
