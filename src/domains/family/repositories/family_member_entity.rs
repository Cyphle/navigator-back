use sqlx::FromRow;

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct FamilyMemberEntity {
    id: i32,
    family_id: i32,
    user_id: i32,
    role: Option<String>
}