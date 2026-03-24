use std::fmt::Debug;

pub trait ApplicationError: Debug {
    fn get_message(&self) -> String;
}
