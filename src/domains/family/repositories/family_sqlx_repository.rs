use async_trait::async_trait;
use sqlx::{Error, Postgres, Transaction};
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::repositories::family_entity::FamilyEntity;
use crate::domains::family::repositories::family_repository::FamilyRepository;

pub struct SqlxFamilyRepository;

#[async_trait]
impl<'a> FamilyRepository<Transaction<'a, Postgres>> for SqlxFamilyRepository {
    async fn get_families_for(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
    ) -> Result<Vec<FamilyEntity>, Error> {
        let families = sqlx::query_as::<sqlx::Postgres, FamilyEntity>(
            "SELECT f.id, f.name FROM families f \
            INNER JOIN family_members fm ON f.id = fm.family_id \
            INNER JOIN users u ON fm.user_id = u.id \
            WHERE u.username = $1",
        )
            .bind(username)
            .fetch_all(&mut **tx)
            .await?;

        Ok(families)
    }

    async fn get_family_by_name(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
        name: &str,
    ) -> Result<FamilyEntity, Error> {
        let family = sqlx::query_as::<sqlx::Postgres, FamilyEntity>(
            "SELECT f.id, f.name FROM families f \
            INNER JOIN family_members fm ON f.id = fm.family_id \
            INNER JOIN users u ON fm.user_id = u.id \
            WHERE u.username = $1 \
            AND f.name = $2",
        )
            .bind(username)
            .bind(name)
            .fetch_one(&mut **tx)
            .await?;

        Ok(family)
    }

    async fn create_family(&self, tx: &mut Transaction<'a, Postgres>, username: &str, command: CreateFamilyCommand) -> Result<String, Error> {
        let family_id: (i32,) = sqlx::query_as(
            "INSERT INTO families (name) VALUES ($1) RETURNING id",
        )
            .bind(&command.name)
            .fetch_one(&mut **tx)
            .await?;

        let user_id: (i32,) = sqlx::query_as(
            "SELECT id FROM users WHERE username = $1 LIMIT 1",
        )
            .bind(username)
            .fetch_one(&mut **tx)
            .await?;

        sqlx::query(
            "INSERT INTO family_members (family_id, user_id, role) VALUES ($1, $2, $3)",
        )
            .bind(family_id.0)
            .bind(user_id.0)
            .bind(command.role.as_str())
            .execute(&mut **tx)
            .await?;

        Ok(format!("Creation of family {} for username {} done", command.name, username))
    }
}
