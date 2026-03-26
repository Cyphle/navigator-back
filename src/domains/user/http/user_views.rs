use serde::Serialize;

#[derive(Serialize)]
pub struct UserView {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
}