use crate::domains::user::domain::create_user_command::CreateUserCommand;
use crate::domains::user::domain::user::User;
use crate::domains::user::domain::user_repository::UserRepository;
use async_trait::async_trait;
use sqlx::{FromRow, Postgres, Transaction};

#[derive(Debug, FromRow)]
pub struct UserEntity {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
}

pub struct SqlxUserRepository;

#[async_trait]
impl<'a> UserRepository<Transaction<'a, Postgres>> for SqlxUserRepository {
    // Create user
    async fn create_user(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        user: &CreateUserCommand,
    ) -> Result<User, sqlx::Error> {
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

        Ok(User {
            id: row.0,
            username: user.username.clone(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
        })
    }

    // Get user
    async fn get_user(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
    ) -> Result<User, sqlx::Error> {
        // Postgres uses positional parameters like $1
        let row = sqlx::query_as::<Postgres, UserEntity>(
            "SELECT id, username, email, first_name, last_name FROM users WHERE username = $1 LIMIT 1",
        )
            .bind(username)
            .fetch_one(&mut **tx)
            .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email.unwrap_or("".to_string()),
            first_name: row.first_name,
            last_name: row.last_name,
        })
    }

    // Upsert user
    async fn get_or_create_user(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        user: &User,
    ) -> Result<User, sqlx::Error> {
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

        Ok(User {
            id: user.id,
            username: user.username,
            email: user.email.unwrap_or("".to_string()),
            first_name: user.first_name,
            last_name: user.last_name,
        })
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
        let command = CreateUserCommand {
            username: "ignored".to_string(),
            email: "ignored".to_string(),
            first_name: "ignored".to_string(),
            last_name: "ignored".to_string(),
            password: "ignored".to_string(),
        };

        let result = repo.create_user(&mut tx, &command).await.unwrap();
        assert_eq!(result.id, 42);
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
            id: 9,
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
