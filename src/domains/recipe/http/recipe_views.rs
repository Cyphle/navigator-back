use serde::Serialize;

#[derive(Serialize)]
pub struct RecipeSummaryView {
    pub id: i32,
    pub name: String,
    pub favorite: bool,
    pub selected_for_week: bool,
    pub visibility: String,
}