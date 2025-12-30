#[derive(Debug)]
pub enum ApplicationErrors {
    MissingUsername,
    Database(sqlx::Error),
}