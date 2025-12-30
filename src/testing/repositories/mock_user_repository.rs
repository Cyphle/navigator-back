use actix_web::http::StatusCode;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use crate::domain::user::user::User;
use crate::repositories::user::{UserEntity, UserRepository};
use crate::testing::repositories::mock_database::MockTransaction;

pub struct MockUserRepository {
    pub fixed_id: u64,
    pub fixed_username: String,
    pub should_error: bool,
}

impl Default for MockUserRepository {
    fn default() -> Self {
        Self {
            fixed_id: 1,
            fixed_username: "mock_user".to_string(),
            should_error: false,
        }
    }
}

impl MockUserRepository {
    pub fn fixed_entity(&self) -> UserEntity {
        UserEntity {
            id: self.fixed_id as i32,
            username: self.fixed_username.clone(),
        }
    }

    pub fn fixed_create_response(&self) -> (u64, StatusCode) {
        (self.fixed_id, StatusCode::CREATED)
    }
}

#[async_trait]
impl UserRepository<MockTransaction> for MockUserRepository {
    async fn create_user(
        &self,
        _tx: &mut MockTransaction,
        _user: &User,
    ) -> Result<(u64, StatusCode), sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.fixed_create_response())
        }
    }

    async fn get_user(
        &self,
        _tx: &mut MockTransaction,
        _username: &str,
    ) -> Result<UserEntity, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.fixed_entity())
        }
    }

    async fn get_or_create_user(
        &self,
        _tx: &mut MockTransaction,
        _user: &User,
    ) -> Result<UserEntity, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.fixed_entity())
        }
    }
}

#[async_trait]
impl<'a> UserRepository<Transaction<'a, Postgres>> for MockUserRepository {
    async fn create_user(
        &self,
        _tx: &mut Transaction<'a, Postgres>,
        _user: &User,
    ) -> Result<(u64, StatusCode), sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.fixed_create_response())
        }
    }

    async fn get_user(
        &self,
        _tx: &mut Transaction<'a, Postgres>,
        _username: &str,
    ) -> Result<UserEntity, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.fixed_entity())
        }
    }

    async fn get_or_create_user(
        &self,
        _tx: &mut Transaction<'a, Postgres>,
        _user: &User,
    ) -> Result<UserEntity, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.fixed_entity())
        }
    }
}
