use async_trait::async_trait;
use sqlx::{Error, FromRow, Postgres, Transaction};

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
}
