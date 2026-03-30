use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::Family;
use crate::domains::family::domain::family_repository::FamilyRepository;
use crate::testing::repositories::mock_database::MockTransaction;
use async_trait::async_trait;

pub struct MockFamilyRepository {
    pub families: Vec<Family>,
    pub should_error: bool,
}

#[async_trait]
impl FamilyRepository<MockTransaction> for MockFamilyRepository {
    async fn get_families_for(
        &self,
        _tx: &mut MockTransaction,
        _username: &str,
    ) -> Result<Vec<Family>, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.families.clone())
        }
    }

    async fn get_family_by_name(
        &self,
        _tx: &mut MockTransaction,
        _username: &str,
        _name: &str,
    ) -> Result<Family, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else if self.families.is_empty() {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.families[0].clone())
        }
    }

    async fn create_family(
        &self,
        _tx: &mut MockTransaction,
        _username: &str,
        _command: &CreateFamilyCommand,
    ) -> Result<Family, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok(self.families[0].clone())
        }
    }
}
