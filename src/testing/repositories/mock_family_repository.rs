use async_trait::async_trait;
use crate::repositories::family::{FamilyEntity, FamilyRepository};
use crate::testing::repositories::mock_database::MockTransaction;

pub struct MockFamilyRepository {
    pub families: Vec<FamilyEntity>,
}

#[async_trait]
impl FamilyRepository<MockTransaction> for MockFamilyRepository {
    async fn get_family_by_member_username(
        &self,
        _tx: &mut MockTransaction,
        _username: &str,
    ) -> Result<Vec<FamilyEntity>, sqlx::Error> {
        Ok(self.families.clone())
    }
}