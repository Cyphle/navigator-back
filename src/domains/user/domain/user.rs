#[derive(Debug)]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}