use serde::Serialize;

#[derive(serde::Deserialize, Serialize, Clone)]
pub struct CreateFamilyRequest {
    pub name: String,
    #[serde(rename = "creatorRelation")]
    pub creator_relation: String,
    pub members: Vec<CreateFamilyMemberRequest>
}

#[derive(serde::Deserialize, Serialize, Clone)]
pub struct CreateFamilyMemberRequest {
    pub username: String,
    pub relation: String,
    pub is_admin: bool,
}
