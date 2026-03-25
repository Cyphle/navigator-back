use serde::Serialize;

#[derive(Serialize)]
pub struct ShoppingListSummaryView {
    pub id: i32,
    pub name: String,
    pub type_: ShoppingListType,
    pub family_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
enum ShoppingListType {
    Shared,
    Personal,
}