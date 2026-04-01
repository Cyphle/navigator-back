use crate::config::actix::AsPgConn;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::Family;
use async_trait::async_trait;
use sqlx::PgConnection;

#[async_trait]
pub trait FamilyRepository<Tx>: Send + Sync {
    async fn get_families_for(
        &self,
        tx: &mut Tx,
        username: &str,
    ) -> Result<Vec<Family>, sqlx::Error>;

    async fn get_family_by_name(
        &self,
        tx: &mut Tx,
        username: &str,
        name: &str,
    ) -> Result<Family, sqlx::Error>;

    async fn create_family(
        &self,
        tx: &mut Tx,
        username: &str,
        command: &CreateFamilyCommand
    ) -> Result<Family, sqlx::Error>;
}

// For generic erasure (object safe)
#[async_trait]
pub trait DynFamilyRepository: Send + Sync {
    async fn get_families_for(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
    ) -> Result<Vec<Family>, sqlx::Error>;

    async fn get_family_by_name(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        name: &str,
    ) -> Result<Family, sqlx::Error>;

    async fn create_family(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        command: &CreateFamilyCommand
    ) -> Result<Family, sqlx::Error>;
}
