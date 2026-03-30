use serde::Serialize;

#[derive(Serialize)]
pub struct KeycloakUser {
    pub username: String,
    pub email: String,
    pub enabled: bool,
    pub credentials: Vec<KeycloakCredential>,
}

#[derive(Serialize)]
pub struct KeycloakCredential {
    pub r#type: String,
    pub value: String,
    pub temporary: bool,
}