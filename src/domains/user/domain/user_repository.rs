use crate::config::actix::AsPgConn;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::user::domain::create_user_command::CreateUserCommand;
use crate::domains::user::domain::user::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, conn: &mut dyn AsPgConn, user: &CreateUserCommand) -> Result<User, RepositoryError>;
    async fn get_user(&self, conn: &mut dyn AsPgConn, username: &str) -> Result<User, RepositoryError>;
    async fn get_or_create_user(&self, conn: &mut dyn AsPgConn, user: &User) -> Result<User, RepositoryError>;
}
