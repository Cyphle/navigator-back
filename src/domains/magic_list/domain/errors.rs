use crate::domains::common::errors::repository_error::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum CheckMagicListAccessError {
    #[error("magic list not found (id={magic_list_id})")]
    NotFound { magic_list_id: i32 },

    #[error("access denied to magic list (id={magic_list_id})")]
    AccessDenied { magic_list_id: i32 },

    #[error("repository failure while checking access (magic_list_id={magic_list_id})")]
    Repository {
        magic_list_id: i32,
        #[source]
        source: RepositoryError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CreateMagicListError {
    #[error("repository failure while creating magic list (name={name})")]
    Repository {
        name: String,
        #[source]
        source: RepositoryError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum GetMagicListSummaryError {
    #[error("repository failure while fetching summaries (username={username}, family_id={family_id})")]
    Repository {
        username: String,
        family_id: i32,
        #[source]
        source: RepositoryError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum AddItemToMagicListError {
    #[error("access check failed (magic_list_id={magic_list_id})")]
    Access {
        magic_list_id: i32,
        #[source]
        source: CheckMagicListAccessError,
    },

    #[error("repository failure while adding item (magic_list_id={magic_list_id})")]
    Repository {
        magic_list_id: i32,
        #[source]
        source: RepositoryError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateItemOfMagicListError {
    #[error("access check failed (magic_list_id={magic_list_id})")]
    Access {
        magic_list_id: i32,
        #[source]
        source: CheckMagicListAccessError,
    },

    #[error("repository failure while updating item (magic_list_id={magic_list_id}, item_id={item_id})")]
    Repository {
        magic_list_id: i32,
        item_id: i32,
        #[source]
        source: RepositoryError,
    },
}
