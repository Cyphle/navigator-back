use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Visibility {
    Shared,
    Personal,
}