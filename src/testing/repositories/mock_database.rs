use std::pin::Pin;
use crate::config::actix::DbConnection;

pub struct MockPoolPostgres;

pub struct MockTransaction;

impl DbConnection for MockPoolPostgres {
    type Tx<'a> = MockTransaction;

    fn begin<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
        Box::pin(async { Ok(MockTransaction) })
    }
}