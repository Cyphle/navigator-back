use crate::domains::common::errors::repository_error::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum CreateFamilyError {
    #[error("family already exists: {name}")]
    AlreadyExists { name: String },

    #[error("repository failure while creating family (name={name})")]
    Repository {
        name: String,
        #[source]
        source: RepositoryError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum GetFamiliesError {
    #[error("repository failure while fetching families (username={username})")]
    Repository {
        username: String,
        #[source]
        source: RepositoryError,
    },
}
