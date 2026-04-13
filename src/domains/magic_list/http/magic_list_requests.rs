use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateMagicListRequest {
    pub name: String,
    pub visibility: String,
    pub magic_list_type: String,
    pub family_id: Option<i32>,
    pub excluded_member_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMagicListItemRequest {
    pub title: String,
    pub content: Option<String>,
    pub checked: Option<bool>,
    pub due_date: Option<String>,
    pub status: Option<String>,
}
