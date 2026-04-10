use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum MagicListType {
    Simple,
    Task,
    Template,
}

impl MagicListType {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TASK" => MagicListType::Task,
            "TEMPLATE" => MagicListType::Template,
            _ => MagicListType::Simple,
        }
    }
}

impl Display for MagicListType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagicListType::Simple => write!(f, "SIMPLE"),
            MagicListType::Task => write!(f, "TASK"),
            MagicListType::Template => write!(f, "TEMPLATE"),
        }
    }
}
