use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub code: Option<String>,
    #[allow(dead_code)]
    session_state: Option<String>,
    #[allow(dead_code)]
    iss: Option<String>,
}
