#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyRole {
    Owner,
}

impl FamilyRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            FamilyRole::Owner => "OWNER",
        }
    }
}
