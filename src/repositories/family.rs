use async_trait::async_trait;
use sqlx::{Error, FromRow, Postgres, Transaction};
use crate::domain::family::family::CreateFamilyCommand;

#[derive(Debug, FromRow, Clone)]
pub struct FamilyEntity {
    #[allow(dead_code)]
    pub id: i32,
    pub name: String,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct FamilyMember {
    id: i32,
    family_id: i32,
    user_id: i32,
    role: Option<String>
}

#[async_trait]
pub trait FamilyRepository<Tx>: Send + Sync {
    async fn get_families_for(
        &self,
        tx: &mut Tx,
        username: &str,
    ) -> Result<Vec<FamilyEntity>, sqlx::Error>;

    async fn get_family_by_name(
        &self,
        tx: &mut Tx,
        username: &str,
        name: &str,
    ) -> Result<FamilyEntity, sqlx::Error>;

    async fn create_family(
        &self,
        tx: &mut Tx,
        username: &str,
        command: CreateFamilyCommand
    ) -> Result<String, sqlx::Error>;
}

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
