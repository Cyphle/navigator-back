use crate::domains::common::visibility::Visibility;

#[derive(Debug, Clone)]
pub struct MagicList {
    pub id: i32,
    pub owner_username: String,
    pub visibility: Visibility,
    pub family_id: Option<i32>,
}
