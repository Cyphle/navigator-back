use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub code: Option<String>,
    session_state: Option<String>,
    iss: Option<String>,
}