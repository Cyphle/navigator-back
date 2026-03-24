use crate::domains::family::domain::family_role::FamilyRole;

pub struct CreateFamilyCommand {
    pub name: String,
    pub role: FamilyRole,
}