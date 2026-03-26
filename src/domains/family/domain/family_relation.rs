#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyRelation {
    Parent,
    GrandParent,
    Child,
    Uncle,
    Aunt,
    Sister,
    Brother,
    Other,
}

impl FamilyRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            FamilyRelation::Parent => "PARENT",
            FamilyRelation::GrandParent => "GRAND_PARENT",
            FamilyRelation::Child => "CHILD",
            FamilyRelation::Uncle => "UNDLE",
            FamilyRelation::Aunt => "AUNT",
            FamilyRelation::Sister => "SISTER",
            FamilyRelation::Brother => "BROTHER",
            FamilyRelation::Other => "OTHER"
        }
    }
}

pub fn from_str(relation: &String) -> FamilyRelation {
    match relation.as_str() {
        "PARENT" => FamilyRelation::Parent,
        "GRAND_PARENT" => FamilyRelation::GrandParent,
        "CHILD" => FamilyRelation::Child,
        "UNDLE" => FamilyRelation::Uncle,
        "AUNT" => FamilyRelation::Aunt,
        "SISTER" => FamilyRelation::Sister,
        "BROTHER" => FamilyRelation::Brother,
        _ => FamilyRelation::Other
    }
}
