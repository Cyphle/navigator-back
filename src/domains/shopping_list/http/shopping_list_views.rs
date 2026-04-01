use serde::Serialize;
use crate::domains::common::visibility::Visibility;

#[derive(Serialize)]
pub struct ShoppingListSummaryView {
    pub id: i32,
    pub name: String,
    pub type_: Visibility,
    pub family_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}
