use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum MagicListItemStatus {
    Todo,
    InProgress,
    Done,
}

impl MagicListItemStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TODO" => Some(MagicListItemStatus::Todo),
            "IN_PROGRESS" => Some(MagicListItemStatus::InProgress),
            "DONE" => Some(MagicListItemStatus::Done),
            _ => None,
        }
    }
}

impl Display for MagicListItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagicListItemStatus::Todo => write!(f, "TODO"),
            MagicListItemStatus::InProgress => write!(f, "IN_PROGRESS"),
            MagicListItemStatus::Done => write!(f, "DONE"),
        }
    }
}
