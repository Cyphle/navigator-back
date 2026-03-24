use serde::Serialize;

#[derive(Serialize)]
pub struct TodoSummaryView {
    pub id: String,
    pub label: String,
    pub assignee: String,
    pub completed: bool,
    pub visibility: String,
}
