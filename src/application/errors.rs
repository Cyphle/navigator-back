#[derive(Debug)]
pub enum ApplicationErrors {
    FamilyAlreadyExists,
    MissingUsername,
    Database(sqlx::Error),
}
