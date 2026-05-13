use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found")]
    NotFound,

    #[error("conflict on {0}")]
    Conflict(String),

    #[error("technical error in repository")]
    Technical(#[source] Box<dyn Error + Send + Sync + 'static>),
}

impl RepositoryError {
    pub fn technical<E: Error + Send + Sync + 'static>(e: E) -> Self {
        Self::Technical(Box::new(e))
    }
}

// Automatically map SQLx errors to RepositoryError
impl From<sqlx::Error> for RepositoryError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                RepositoryError::Conflict(db.constraint().unwrap_or("unknown").to_string())
            }
            _ => RepositoryError::Technical(Box::new(e)),
        }
    }
}
