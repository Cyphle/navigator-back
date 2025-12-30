use async_trait::async_trait;
use crate::domain::user::user::User;
use crate::repositories::user::UserRepository;
use crate::testing::repositories::mock_database::MockTransaction;

pub struct MockUserRepository;

#[async_trait]
impl UserRepository<MockTransaction> for MockUserRepository {
    async fn create_user(
        &self,
        _tx: &mut MockTransaction,
        _user: &User,
    ) -> Result<(u64, actix_web::http::StatusCode), sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }

    async fn get_user(
        &self,
        _tx: &mut MockTransaction,
        _username: &str,
    ) -> Result<crate::repositories::user::UserEntity, sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }

    async fn get_or_create_user(
        &self,
        _tx: &mut MockTransaction,
        _user: &User,
    ) -> Result<crate::repositories::user::UserEntity, sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }
}