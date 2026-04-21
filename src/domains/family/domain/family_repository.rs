use crate::config::actix::AsPgConn;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::Family;
use async_trait::async_trait;

#[async_trait]
pub trait FamilyRepository: Send + Sync {
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

    async fn is_family_member(
        &self,
        username: &str,
        family_id: i32,
    ) -> Result<bool, sqlx::Error>;
}
