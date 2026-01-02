use serde::Serialize;

#[derive(serde::Deserialize, Serialize)]
pub struct CreateFamilyRequest {
    pub name: String,
}