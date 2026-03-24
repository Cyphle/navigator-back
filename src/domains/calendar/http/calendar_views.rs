use serde::Serialize;

#[derive(Serialize)]
pub struct CalendarSummaryView {
    pub id: String,
    pub title: String,
    pub time: String,
    pub person: String,
    pub calendar_color: String,
    pub visibility: String,
    pub attendees: Vec<String>,
}
