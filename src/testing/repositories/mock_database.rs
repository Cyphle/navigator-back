use std::pin::Pin;
use crate::config::actix::{DbConnection, DbTransaction};

pub struct MockPoolPostgres;
pub struct MockPoolPostgresError;

pub struct MockTransaction;

// TODO à revoir tous ces mocks. c'est difficile à comprendre
impl DbTransaction for MockTransaction {
    fn commit<'a>(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    where
        Self: 'a,
    {
        Box::pin(async { Ok(()) })
    }

    fn rollback<'a>(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    where
        Self: 'a,
    {
        Box::pin(async { Ok(()) })
    }
}

impl DbConnection for MockPoolPostgres {
    type Tx<'a> = MockTransaction;

    fn begin<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
        Box::pin(async { Ok(MockTransaction) })
    }
}

impl DbConnection for MockPoolPostgresError {
    type Tx<'a> = MockTransaction;

    fn begin<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
        Box::pin(async { Err(sqlx::Error::RowNotFound) })
    }
}
