use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MagicListSummaryView {
    pub id: i32,
    pub name: String,
    pub visibility: String,
    #[serde(rename = "type")]
    pub magic_list_type: String,
    pub family_id: Option<i32>,
    pub item_count: i64,
}

impl From<MagicListSummary> for MagicListSummaryView {
    fn from(summary: MagicListSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            visibility: summary.visibility.to_string(),
            magic_list_type: summary.magic_list_type.to_string(),
            family_id: summary.family_id,
            item_count: summary.item_count,
        }
    }
}
