use crate::domains::common::errors::errors::ApplicationError;
use std::fmt::Debug;

#[derive(Debug)]
pub struct RepositoryError {
    pub error: String
}

impl ApplicationError for RepositoryError {
    fn get_message(&self) -> String {
        self.error.clone()
    }
}
