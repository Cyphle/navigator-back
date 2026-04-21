use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::{Family, FamilyMember};
use crate::config::actix::AsPgConn;
use crate::domains::family::domain::family_repository::FamilyRepository;
use async_trait::async_trait;

pub struct MockFamilyRepository {
    pub families: Vec<Family>,
    pub should_error: bool,
    pub is_family_member: bool,
}

#[async_trait]
impl FamilyRepository for MockFamilyRepository {
    async fn get_families_for(&self, _conn: &mut dyn AsPgConn, _username: &str) -> Result<Vec<Family>, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(self.families.clone())
    }

    async fn get_family_by_name(&self, _conn: &mut dyn AsPgConn, username: &str, name: &str) -> Result<Family, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        self.families
            .iter()
            .find(|f| f.name == name && f.creator_username == username)
            .cloned()
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn is_family_member(&self, _username: &str, _family_id: i32) -> Result<bool, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(self.is_family_member)
    }

    async fn create_family(&self, _conn: &mut dyn AsPgConn, username: &str, command: &CreateFamilyCommand) -> Result<Family, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(Family {
            id: (self.families.len() + 1) as i32,
            name: command.name.clone(),
            creator_username: username.to_string(),
            members: command.members.iter().map(|m| FamilyMember {
                username: m.username_or_email.clone(),
                relation: m.relation.clone(),
                is_admin: m.is_admin,
            }).collect(),
            active: true,
        })
    }
}
