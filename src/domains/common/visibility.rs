use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Visibility {
    Shared,
    Personal,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Shared => write!(f, "SHARED"),
            Visibility::Personal => write!(f, "PERSONAL"),
        }
    }
}