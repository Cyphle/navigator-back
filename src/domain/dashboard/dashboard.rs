#[derive(Debug, PartialEq)]
pub struct Dashboard {
    agenda: Vec<String>,
    todos: Vec<String>,
    weeklyMenu: String,
    recipes: Vec<String>,
    shopping: String
}

pub fn empty() -> Dashboard {
    Dashboard {
        agenda: Vec::new(),
        todos: Vec::new(),
        weeklyMenu: "".to_string(),
        recipes: Vec::new(),
        shopping: "".to_string()
    }
}