use crate::domains::family::domain::family_repository::FamilyRepository;
use crate::domains::user::domain::user_repository::UserRepository;
use crate::security::oidc::OidcConfig;
use openid::{Client, Discovered, StandardClaims};
use sqlx::{PgConnection, Pool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use crate::domains::bank_account::domain::bank_account_repository::BankAccountRepository;
use crate::domains::magic_list::domain::magic_list_repository::MagicListRepository;

pub trait AsPgConn: Send {
    fn as_pg_conn(&mut self) -> &mut PgConnection;
}

impl<'a> AsPgConn for Transaction<'a, Postgres> {
    fn as_pg_conn(&mut self) -> &mut PgConnection {
        &mut **self
    }
}

pub trait DbConnection: Send + Sync {
    type Tx<'a>: DbTransaction + Send
    where
        Self: 'a;

    fn begin<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>>;
}

pub trait DbTransaction: Send {
    fn commit<'a>(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    where
        Self: 'a;

    fn rollback<'a>(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    where
        Self: 'a;
}

impl<'a> DbTransaction for Transaction<'a, Postgres> {
    fn commit<'b>(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'b>>
    where
        Self: 'b,
    {
        Box::pin(async move { self.commit().await })
    }

    fn rollback<'b>(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'b>>
    where
        Self: 'b,
    {
        Box::pin(async move { self.rollback().await })
    }
}

impl DbConnection for Pool<Postgres> {
    type Tx<'a> = Transaction<'a, Postgres>;

    fn begin<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
        Box::pin(async move { self.begin().await })
    }
}

impl AsPgConn for Pool<Postgres> {
    fn as_pg_conn(&mut self) -> &mut PgConnection {
        unimplemented!("Pool cannot be converted to PgConnection directly, use a transaction or acquire a connection")
    }
}

pub struct ActixState<DB = Pool<Postgres>>
where
    DB: DbConnection,
{
    pub oidc_config: OidcConfig,
    pub oidc_client: Option<Arc<Mutex<Client<Discovered, StandardClaims>>>>,

    pub db_connection: DB,
    pub user_repository: Arc<dyn UserRepository>,
    pub family_repository: Arc<dyn FamilyRepository>,
    pub bank_account_repository: Arc<dyn BankAccountRepository>,
    pub magic_list_repository: Arc<dyn MagicListRepository>,
}
