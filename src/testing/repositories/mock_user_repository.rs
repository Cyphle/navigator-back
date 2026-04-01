use crate::config::actix::AsPgConn;
use crate::domains::user::domain::create_user_command::CreateUserCommand;
use crate::domains::user::domain::user::User;
use crate::domains::user::domain::user_repository::UserRepository;
use async_trait::async_trait;

pub struct MockUserRepository {
    pub fixed_id: u64,
    pub fixed_username: String,
    pub fixed_email: String,
    pub fixed_first_name: String,
    pub fixed_last_name: String,
    pub should_error: bool,
}

impl Default for MockUserRepository {
    fn default() -> Self {
        Self {
            fixed_id: 1,
            fixed_username: "mock_user".to_string(),
            fixed_email: "mock_email".to_string(),
            fixed_first_name: "mock_first_name".to_string(),
            fixed_last_name: "mock_last_name".to_string(),
            should_error: false,
        }
    }
}

impl MockUserRepository {
    pub fn fixed_user(&self) -> User {
        User {
            id: self.fixed_id as i32,
            username: self.fixed_username.clone(),
            email: self.fixed_email.clone(),
            first_name: self.fixed_first_name.clone(),
            last_name: self.fixed_last_name.clone(),
        }
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create_user(&self, _conn: &mut dyn AsPgConn, _user: &CreateUserCommand) -> Result<User, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(self.fixed_user())
    }
    async fn get_user(&self, _conn: &mut dyn AsPgConn, _username: &str) -> Result<User, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(self.fixed_user())
    }
    async fn get_or_create_user(&self, _conn: &mut dyn AsPgConn, _user: &User) -> Result<User, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(self.fixed_user())
    }
}

