use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::{Family, FamilyMember};
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
        username: &str,
        name: &str,
    ) -> Result<Family, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            self.families
                .iter()
                .find(|f| f.name == name && f.creator_username == username)
                .cloned()
                .ok_or(sqlx::Error::RowNotFound)
        }
    }

    async fn create_family(
        &self,
        _tx: &mut MockTransaction,
        username: &str,
        command: &CreateFamilyCommand,
    ) -> Result<Family, sqlx::Error> {
        if self.should_error {
            Err(sqlx::Error::RowNotFound)
        } else {
            let family = Family {
                id: (self.families.len() + 1) as i32,
                name: command.name.clone(),
                creator_username: username.to_string(),
                members: command
                    .members
                    .iter()
                    .map(|m| FamilyMember {
                        username: m.username_or_email.clone(),
                        relation: m.relation.clone(),
                        is_admin: m.is_admin,
                    })
                    .collect(),
                active: true,
            };
            Ok(family)
        }
    }
}
