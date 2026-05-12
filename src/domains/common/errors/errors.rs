use std::fmt::Debug;

pub trait ApplicationError: Debug {
    fn get_message(&self) -> String;
    fn status_code(&self) -> u16 { 500 }
}