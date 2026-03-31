use crate::domains::user::domain::create_user_command::CreateUserCommand;
use crate::domains::user::domain::user::User;
use async_trait::async_trait;
use sqlx::PgConnection;

#[async_trait]
pub trait UserRepository<Tx>: Send + Sync {
    async fn create_user(
        &self,
        tx: &mut Tx,
        user: &CreateUserCommand,
    ) -> Result<User, sqlx::Error>;
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

// For generic erasure (object safe)
#[async_trait]
pub trait DynUserRepository: Send + Sync {
    async fn create_user(&self, conn: &mut PgConnection, user: &CreateUserCommand) -> Result<User, sqlx::Error>;
    async fn get_user(&self, conn: &mut PgConnection, username: &str) -> Result<User, sqlx::Error>;
    async fn get_or_create_user(&self, conn: &mut PgConnection, user: &User) -> Result<User, sqlx::Error>;
}
