use actix_web::http::StatusCode;
use async_trait::async_trait;
use sqlx::{FromRow, Postgres, Transaction};
use crate::domain::user::user::User;

#[derive(Debug, FromRow)]
pub struct UserEntity {
    id: i32,
    pub username: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user: &User,
    ) -> Result<(u64, StatusCode), sqlx::Error>;
    async fn get_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        username: &str,
    ) -> Result<UserEntity, sqlx::Error>;
    async fn get_or_create_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user: &User,
    ) -> Result<UserEntity, sqlx::Error>;
}

pub struct SqlxUserRepository;

#[async_trait]
impl UserRepository for SqlxUserRepository {
    // Create user
    async fn create_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user: &User,
    ) -> Result<(u64, StatusCode), sqlx::Error> {
        let row: (i32,) = sqlx::query_as("INSERT INTO users (username) VALUES ($1) RETURNING id")
            .bind(&user.username)
            .fetch_one(&mut **tx)
            .await?;

        Ok((row.0 as u64, StatusCode::CREATED))
    }

    // Get user
    async fn get_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        username: &str,
    ) -> Result<UserEntity, sqlx::Error> {
        // Postgres uses positional parameters like $1
        let row = sqlx::query_as::<sqlx::Postgres, UserEntity>(
            "SELECT id FROM users WHERE username = $1 LIMIT 1",
        )
            .bind(username)
            .fetch_one(&mut **tx)
            .await?;

        Ok(row)
    }

    // Upsert user
    async fn get_or_create_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user: &User,
    ) -> Result<UserEntity, sqlx::Error> {
        let user = sqlx::query_as::<Postgres, UserEntity>(
            r#"
        INSERT INTO users (username)
        VALUES ($1)
        ON CONFLICT (username)
        DO UPDATE SET username = EXCLUDED.username
        RETURNING id, username
        "#
        )
            .bind(&user.username)
            .fetch_one(&mut **tx)
            .await?;

        Ok(user)
    }
}
