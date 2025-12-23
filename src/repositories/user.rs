use crate::domain::user::User;
use actix_web::http::StatusCode;
use sqlx::{Database, Executor, FromRow, Pool, Postgres};

#[derive(Debug, FromRow)]
struct UserEntity {
    id: i32,
    username: String,
}

// Create user
async fn create_user<'e, E>(pool: E, user: &User) -> Result<(u64, StatusCode), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let row: (i32,) = sqlx::query_as("INSERT INTO users (username) VALUES ($1) RETURNING id")
        .bind(&user.username)
        .fetch_one(pool)
        .await?;

    Ok((row.0 as u64, StatusCode::CREATED))
}

// Get user
async fn get_user<'e, E>(pool: &Pool<Postgres>, username: String) -> Result<UserEntity, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    // Postgres uses positional parameters like $1
    let row = sqlx::query_as::<sqlx::Postgres, UserEntity>(
        "SELECT id FROM users WHERE username = $1 LIMIT 1",
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

// Upsert user
pub async fn get_or_create_user<'e, E>(
    exec: E,
    user: &User,
) -> Result<UserEntity, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
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
        .fetch_one(exec)
        .await?;

    Ok(user)
}