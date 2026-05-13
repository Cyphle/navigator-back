#[derive(Debug, thiserror::Error)]
#[error("No username_or_email specified")]
pub struct MissingUsernameError;
