use serde::Serialize;

#[derive(serde::Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateFamilyRequest {
    pub name: String,
    pub creator_relation: String,
    pub members: Vec<CreateFamilyMemberRequest>
}

#[derive(serde::Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateFamilyMemberRequest {
    #[serde(alias = "username")]
    pub username_or_email: String,
    pub relation: String,
    pub is_admin: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize_create_family_request() {
        let json = r#"{
          "name": "Ma famille",
          "creatorRelation": "PARENT",
          "members": [
            {
              "username": "toto@mafamille.com",
              "relation": "CHILD",
              "isAdmin": false
            },
            {
              "username": "maman@mafamille.com",
              "relation": "PARENT",
              "isAdmin": true
            }
          ]
        }"#;

        let request: CreateFamilyRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.name, "Ma famille");
        assert_eq!(request.creator_relation, "PARENT");
        assert_eq!(request.members.len(), 2);
        assert_eq!(request.members[0].username_or_email, "toto@mafamille.com");
        assert_eq!(request.members[0].relation, "CHILD");
        assert_eq!(request.members[0].is_admin, false);
        assert_eq!(request.members[1].username_or_email, "maman@mafamille.com");
        assert_eq!(request.members[1].relation, "PARENT");
        assert_eq!(request.members[1].is_admin, true);
    }
}
