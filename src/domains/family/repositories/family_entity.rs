use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct FamilyEntity {
    #[allow(dead_code)]
    pub id: i32,
    pub name: String,
    pub creator_id: i32,
    pub active: bool,
    pub user_id: i32,
    pub username: String,
    pub relation: String,
    pub is_admin: bool
}
