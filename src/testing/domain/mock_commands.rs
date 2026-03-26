use crate::domains::family::domain::create_family_command::CreateFamilyMemberCommand;
use crate::domains::family::domain::family_relation::FamilyRelation;

pub struct MockFamilyCommand {
    pub name: String,
    creator_relation: Option<FamilyRelation>,
    members: Vec<CreateFamilyMemberCommand>
}

impl MockFamilyCommand {
    pub fn new(name: String) -> Self {
        Self {
            name,
            creator_relation: None,
            members: vec![]
        }
    }

    pub fn add_creator_relation(self, relation: FamilyRelation) -> Self {
        Self {
            creator_relation: Some(relation),
            ..self
        }
    }

    pub fn add_member(self, member: CreateFamilyMemberCommand) -> Self {
        Self {
            members: self.members.into_iter().chain(std::iter::once(member)).collect(),
            ..self
        }
    }
}