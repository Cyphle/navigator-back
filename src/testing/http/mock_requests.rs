use crate::domains::family::http::family_requests::{CreateFamilyMemberRequest, CreateFamilyRequest};

pub struct MockFamilyRequest {
    pub name: String,
    creator_relation: Option<String>,
    members: Vec<CreateFamilyMemberRequest>
}

impl MockFamilyRequest {
    pub fn new(name: String) -> Self {
        Self {
            name,
            creator_relation: None,
            members: vec![]
        }
    }

    pub fn add_creator_relation(self, relation: String) -> Self {
        Self {
            creator_relation: Some(relation),
            ..self
        }
    }

    pub fn add_member(self, member: CreateFamilyMemberRequest) -> Self {
        Self {
            members: self.members.into_iter().chain(std::iter::once(member)).collect(),
            ..self
        }
    }

    pub fn build(&self) -> CreateFamilyRequest {
        CreateFamilyRequest {
            name: self.name.clone(),
            creator_relation: self.creator_relation.clone().unwrap(),
            members: self.members.iter().map(|m| m.clone()).collect()
        }
    }
}