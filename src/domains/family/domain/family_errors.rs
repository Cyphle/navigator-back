#[derive(Debug)]
pub struct FamilyAlreadyExistsError {
    pub name: String
}

impl crate::domains::common::errors::errors::ApplicationError for FamilyAlreadyExistsError {
    fn get_message(&self) -> String {
        format!("Family already exists: {}", self.name)
    }
}
