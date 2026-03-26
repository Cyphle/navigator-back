use crate::domains::family::domain::family_relation::FamilyRelation;

#[derive(Debug)]
pub struct Family {
    pub id: u32,
    pub name: String,
    pub creator_username: String,
    pub members: Vec<FamilyMember>,
    pub active: bool
}

#[derive(Debug)]
pub struct FamilyMember {
    pub username: String,
    pub relation: FamilyRelation,
    pub is_admin: bool,
}

