use chrono::NaiveDate;
use crate::domains::magic_list::domain::magic_list_item_status::MagicListItemStatus;

pub struct UpdateMagicListItemCommand {
    pub title: Option<String>,
    pub content: Option<String>,
    pub checked: Option<bool>,
    pub due_date: Option<NaiveDate>,
    pub status: Option<MagicListItemStatus>,
}
