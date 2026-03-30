use crate::domains::user::domain::user::User;
use actix_web::http::StatusCode;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository<Tx>: Send + Sync {
    async fn create_user(
        &self,
        tx: &mut Tx,
        user: &User,
    ) -> Result<(u64, StatusCode), sqlx::Error>;
    async fn get_user(
        &self,
        tx: &mut Tx,
        username: &str,
    ) -> Result<User, sqlx::Error>;
    async fn get_or_create_user(
        &self,
        tx: &mut Tx,
        user: &User,
    ) -> Result<User, sqlx::Error>;
}