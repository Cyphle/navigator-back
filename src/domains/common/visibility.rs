use serde::Serialize;

#[derive(Serialize)]
pub enum Visibility {
    Shared,
    Personal,
}