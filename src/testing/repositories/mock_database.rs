use std::pin::Pin;
use crate::config::actix::{AsPgConn, DbConnection, DbTransaction};
use sqlx::PgConnection;

#[derive(Clone)]
pub struct MockPoolPostgres;
pub struct MockPoolPostgresError;

pub struct MockTransaction;

impl AsPgConn for MockTransaction {
    fn as_pg_conn(&mut self) -> &mut PgConnection {
        unimplemented!("MockTransaction has no real PgConnection — repo impls should ignore it")
    }
}

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

impl AsPgConn for MockPoolPostgres {
    fn as_pg_conn(&mut self) -> &mut PgConnection {
        unimplemented!("MockPoolPostgres has no real PgConnection")
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
