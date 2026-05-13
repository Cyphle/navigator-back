use crate::domains::common::errors::repository_error::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum GetUserInfoError {
    #[error("repository failure while fetching user info (username={username})")]
    Repository {
        username: String,
        #[source]
        source: RepositoryError,
    },
}
