use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::family::domain::family_errors::{CreateFamilyError, GetFamiliesError};
use crate::domains::magic_list::domain::errors::{
    AddItemToMagicListError, CreateMagicListError, GetMagicListSummaryError,
    UpdateItemOfMagicListError,
};
use crate::domains::user::domain::errors::GetUserInfoError;

// Automatically map domain errors to middleware errors thanks to #[from]
#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("no username_or_email specified")]
    MissingUsername,

    #[error("invalid due_date format, expected YYYY-MM-DD")]
    InvalidDateFormat,

    #[error(transparent)]
    CreateMagicList(#[from] CreateMagicListError),

    #[error(transparent)]
    GetMagicListSummary(#[from] GetMagicListSummaryError),

    #[error(transparent)]
    AddItemToMagicList(#[from] AddItemToMagicListError),

    #[error(transparent)]
    UpdateItemOfMagicList(#[from] UpdateItemOfMagicListError),

    #[error(transparent)]
    CreateFamily(#[from] CreateFamilyError),

    #[error(transparent)]
    GetFamilies(#[from] GetFamiliesError),

    #[error(transparent)]
    GetUserInfo(#[from] GetUserInfoError),
}

impl From<MissingUsernameError> for MiddlewareError {
    fn from(_: MissingUsernameError) -> Self {
        Self::MissingUsername
    }
}
