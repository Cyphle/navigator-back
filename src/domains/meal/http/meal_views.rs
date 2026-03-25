use serde::Serialize;

#[derive(Serialize)]
pub struct MealSummaryView {
    pub week_label: String,
    pub days: Vec<MealDayView>,
}

#[derive(Serialize)]
pub struct MealDayView {
    pub id: i32,
    pub label: String,
    pub entries: Vec<String>,
}

pub fn empty() -> MealSummaryView {
    MealSummaryView {
        week_label: "Mes plats de la semaine".to_string(),
        days: Vec::new(),
    }
}