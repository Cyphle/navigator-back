#[derive(Debug, PartialEq)]
pub struct Dashboard {
    pub agenda: Vec<String>,
    pub todos: Vec<String>,
    pub weeklyMenu: String,
    pub recipes: Vec<String>,
    pub shopping: String
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