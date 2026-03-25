use actix_web::http::StatusCode;
use async_trait::async_trait;
use sqlx::{FromRow, Postgres, Transaction};
use crate::domains::user::domain::user::User;

#[derive(Debug, FromRow)]
pub struct UserEntity {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

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
    ) -> Result<UserEntity, sqlx::Error>;
    async fn get_or_create_user(
        &self,
        tx: &mut Tx,
        user: &User,
    ) -> Result<UserEntity, sqlx::Error>;
}

pub struct SqlxUserRepository;

#[async_trait]
impl<'a> UserRepository<Transaction<'a, Postgres>> for SqlxUserRepository {
    // Create user
    async fn create_user(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        user: &User,
    ) -> Result<(u64, StatusCode), sqlx::Error> {
        let row: (i32,) = sqlx::query_as("
            INSERT INTO users (username, email, first_name, last_name)
            VALUES ($1, $2, $3, $4) RETURNING id
        ")
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .fetch_one(&mut **tx)
            .await?;

        Ok((row.0 as u64, StatusCode::CREATED))
    }

    // Get user
    async fn get_user(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
    ) -> Result<UserEntity, sqlx::Error> {
        // Postgres uses positional parameters like $1
        let row = sqlx::query_as::<sqlx::Postgres, UserEntity>(
            "SELECT id, username, email, first_name, last_name FROM users WHERE username = $1 LIMIT 1",
        )
            .bind(username)
            .fetch_one(&mut **tx)
            .await?;

        Ok(row)
    }

    // Upsert user
    async fn get_or_create_user(
        &self,
        tx: &mut Transaction<'a, Postgres>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::repositories::mock_database::MockTransaction;
    use crate::testing::repositories::mock_user_repository::MockUserRepository;

    #[tokio::test]
    async fn mock_repo_create_user_returns_fixed_values() {
        let repo = MockUserRepository {
            fixed_id: 42,
            fixed_username: "alice".to_string(),
            fixed_email: "alice@example.com".to_string(),
            fixed_first_name: "Alice".to_string(),
            fixed_last_name: "Alicia".to_string(),
            should_error: false,
        };
        let mut tx = MockTransaction;
        let user = User {
            id: Some(42),
            username: "ignored".to_string(),
            email: "ignored".to_string(),
            first_name: "ignored".to_string(),
            last_name: "ignored".to_string(),
        };

        let result = repo.create_user(&mut tx, &user).await.unwrap();
        assert_eq!(result.0, 42);
        assert_eq!(result.1, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn mock_repo_get_user_returns_fixed_entity() {
        let repo = MockUserRepository {
            fixed_id: 7,
            fixed_username: "bob".to_string(),
            fixed_email: "bob@example.com".to_string(),
            fixed_first_name: "Bob".to_string(),
            fixed_last_name: "Bobby".to_string(),
            should_error: false,
        };
        let mut tx = MockTransaction;

        let entity = repo.get_user(&mut tx, "ignored").await.unwrap();
        assert_eq!(entity.id, 7);
        assert_eq!(entity.username, "bob");
    }

    #[tokio::test]
    async fn mock_repo_get_or_create_returns_fixed_entity() {
        let repo = MockUserRepository {
            fixed_id: 9,
            fixed_username: "johndoe".to_string(),
            fixed_email: "johndoe@example.com".to_string(),
            fixed_first_name: "John".to_string(),
            fixed_last_name: "Doe".to_string(),
            should_error: false,
        };
        let mut tx = MockTransaction;
        let user = User {
            id: Some(9),
            username: "ignored".to_string(),
            email: "ignored".to_string(),
            first_name: "ignored".to_string(),
            last_name: "ignored".to_string(),
        };

        let entity = repo.get_or_create_user(&mut tx, &user).await.unwrap();
        assert_eq!(entity.id, 9);
        assert_eq!(entity.username, "johndoe");
    }
}
