use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct FamilyEntity {
    #[allow(dead_code)]
    pub id: i32,
    pub name: String,
}