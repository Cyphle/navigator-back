use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::magic_list_repository::MagicListRepository;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};

pub struct SqlxMagicListRepository {
    pub pool: Pool<Postgres>,
}

#[async_trait]
impl MagicListRepository for SqlxMagicListRepository {
    async fn create(&self, username: &String, command: CreateMagicListCommand) -> Result<(), Box<dyn ApplicationError>> {
        let owner_id: (i32,) = sqlx::query_as(
            "SELECT id FROM users WHERE username = $1",
        )
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        sqlx::query(
            "INSERT INTO magic_list (name, type, visibility, owner_id, family_id, excluded_user_ids) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(command.name)
        .bind(command.magic_list_type.to_string())
        .bind(command.visibility.to_string())
        .bind(owner_id.0)
        .bind(command.family_id)
        .bind(command.excluded_member_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        Ok(())
    }
}
