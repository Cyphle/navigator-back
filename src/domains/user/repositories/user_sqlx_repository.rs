use crate::domains::user::domain::create_user_command::CreateUserCommand;
use crate::domains::user::domain::user::User;
use crate::domains::user::domain::user_repository::{DynUserRepository, UserRepository};
use async_trait::async_trait;
use sqlx::{FromRow, PgConnection, Postgres, Transaction};

#[derive(Debug, FromRow)]
pub struct UserEntity {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
}

pub struct SqlxUserRepository;

impl SqlxUserRepository {
    async fn create_user_inner<'e, E: sqlx::Executor<'e, Database = Postgres>>(
        &self, executor: E, user: &CreateUserCommand,
    ) -> Result<User, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO users (username, email, first_name, last_name)
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .fetch_one(executor)
            .await?;

        Ok(User {
            id: row.0,
            username: user.username.clone(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
        })
    }

    async fn get_user_inner<'e, E: sqlx::Executor<'e, Database = Postgres>>(
        &self, executor: E, username: &str,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<Postgres, UserEntity>(
            "SELECT id, username, email, first_name, last_name FROM users WHERE username = $1 LIMIT 1",
        )
            .bind(username)
            .fetch_one(executor)
            .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email.unwrap_or_default(),
            first_name: row.first_name,
            last_name: row.last_name,
        })
    }

    async fn get_or_create_user_inner<'e, E: sqlx::Executor<'e, Database = Postgres>>(
        &self, executor: E, user: &User,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<Postgres, UserEntity>(
            r#"INSERT INTO users (username, email, first_name, last_name)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (username)
               DO UPDATE SET username = EXCLUDED.username
               RETURNING id, username, email, first_name, last_name"#,
        )
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .fetch_one(executor)
            .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email.unwrap_or_default(),
            first_name: row.first_name,
            last_name: row.last_name,
        })
    }
}

// Trait générique — pour les tests d'intégration
#[async_trait]
impl<'a> UserRepository<Transaction<'a, Postgres>> for SqlxUserRepository {
    async fn create_user(&self, tx: &mut Transaction<'a, Postgres>, user: &CreateUserCommand) -> Result<User, sqlx::Error> {
        self.create_user_inner(&mut **tx, user).await
    }
    async fn get_user(&self, tx: &mut Transaction<'a, Postgres>, username: &str) -> Result<User, sqlx::Error> {
        self.get_user_inner(&mut **tx, username).await
    }
    async fn get_or_create_user(&self, tx: &mut Transaction<'a, Postgres>, user: &User) -> Result<User, sqlx::Error> {
        self.get_or_create_user_inner(&mut **tx, user).await
    }
}

// Trait dyn — pour AppState
#[async_trait]
impl DynUserRepository for SqlxUserRepository {
    async fn create_user(&self, conn: &mut PgConnection, user: &CreateUserCommand) -> Result<User, sqlx::Error> {
        self.create_user_inner(&mut *conn, user).await
    }
    async fn get_user(&self, conn: &mut PgConnection, username: &str) -> Result<User, sqlx::Error> {
        self.get_user_inner(&mut *conn, username).await
    }
    async fn get_or_create_user(&self, conn: &mut PgConnection, user: &User) -> Result<User, sqlx::Error> {
        self.get_or_create_user_inner(&mut *conn, user).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx_testcontainers::test]
    async fn test_create_user(mut conn: sqlx::PgConnection) {
        use sqlx::Connection;
        let repo = SqlxUserRepository;
        let mut tx = conn.begin().await.unwrap();

        let command = CreateUserCommand {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            first_name: "Alice".to_string(),
            last_name: "Alicia".to_string(),
            password: "password123".to_string(),
        };

        let result = repo.create_user(&mut tx, &command).await.unwrap();
        assert_eq!(result.username, "alice");
        assert_eq!(result.email, "alice@example.com");
    }

    #[sqlx_testcontainers::test]
    async fn test_get_user(mut conn: sqlx::PgConnection) {
        use sqlx::Connection;
        let repo = SqlxUserRepository;
        let mut tx = conn.begin().await.unwrap();

        // Seed data
        sqlx::query("INSERT INTO users (username, email, first_name, last_name) VALUES ($1, $2, $3, $4)")
            .bind("bob")
            .bind("bob@example.com")
            .bind("Bob")
            .bind("Bobby")
            .execute(&mut *tx)
            .await
            .unwrap();

        let entity = repo.get_user(&mut tx, "bob").await.unwrap();
        assert_eq!(entity.username, "bob");
        assert_eq!(entity.email, "bob@example.com");
    }

    #[sqlx_testcontainers::test]
    async fn test_get_or_create_user(mut conn: sqlx::PgConnection) {
        use sqlx::Connection;
        let repo = SqlxUserRepository;
        let mut tx = conn.begin().await.unwrap();

        let user = User {
            id: 0,
            username: "johndoe".to_string(),
            email: "".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
        };

        // First call should create
        let entity = repo.get_or_create_user(&mut tx, &user).await.unwrap();
        assert_eq!(entity.username, "johndoe");

        // Second call should return existing
        let entity2 = repo.get_or_create_user(&mut tx, &user).await.unwrap();
        assert_eq!(entity2.id, entity.id);
        assert_eq!(entity2.username, "johndoe");
    }
}
