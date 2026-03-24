use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::family::repositories::family_sqlx_repository::SqlxFamilyRepository;
use crate::domains::user::repositories::user_repository::{SqlxUserRepository, UserRepository};
use crate::security::oidc::OidcConfig;
use openid::{Client, Discovered, StandardClaims};
use sqlx::{Pool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

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

pub struct ActixState<
    DB = Pool<Postgres>,
    U = SqlxUserRepository,
    F = SqlxFamilyRepository,
>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    pub oidc_config: OidcConfig,
    pub oidc_client: Option<Arc<Mutex<Client<Discovered, StandardClaims>>>>,

    pub db_connection: DB,
    pub user_repository: Arc<U>,
    pub family_repository: Arc<F>,
}
