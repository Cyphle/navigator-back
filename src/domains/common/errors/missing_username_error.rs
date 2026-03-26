use crate::domains::common::errors::errors::ApplicationError;
use std::fmt::Debug;

#[derive(Debug)]
pub struct MissingUsernameError;

impl ApplicationError for MissingUsernameError {
    fn get_message(&self) -> String {
        "No username_or_email specified".to_string()
    }
}

