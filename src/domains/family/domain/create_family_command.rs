use crate::domains::family::domain::family_relation::FamilyRelation;

pub struct CreateFamilyCommand {
    pub name: String,
    pub creator_relation: FamilyRelation,
    pub members: Vec<CreateFamilyMemberCommand>
}

pub struct CreateFamilyMemberCommand {
    pub username_or_email: String,
    pub relation: FamilyRelation,
    pub is_admin: bool,
}