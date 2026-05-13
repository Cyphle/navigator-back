use crate::domains::common::errors::middleware_error::MiddlewareError;
use crate::domains::family::domain::family_errors::CreateFamilyError;
use crate::domains::magic_list::domain::errors::{
    AddItemToMagicListError, CheckMagicListAccessError, UpdateItemOfMagicListError,
};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::error::Error;

// Actix convert MiddlewareError to HTTP response
impl ResponseError for MiddlewareError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingUsername => StatusCode::UNAUTHORIZED,
            Self::InvalidDateFormat => StatusCode::BAD_REQUEST,

            Self::AddItemToMagicList(AddItemToMagicListError::Access { source, .. }) => {
                access_error_status(source)
            }
            Self::UpdateItemOfMagicList(UpdateItemOfMagicListError::Access { source, .. }) => {
                access_error_status(source)
            }

            Self::CreateFamily(CreateFamilyError::AlreadyExists { .. }) => StatusCode::CONFLICT,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        log_error_chain(self);

        let body = if status.is_server_error() {
            "internal server error".to_string()
        } else {
            self.to_string()
        };

        HttpResponse::build(status).json(body)
    }
}

fn access_error_status(e: &CheckMagicListAccessError) -> StatusCode {
    match e {
        CheckMagicListAccessError::NotFound { .. } => StatusCode::NOT_FOUND,
        CheckMagicListAccessError::AccessDenied { .. } => StatusCode::FORBIDDEN,
        CheckMagicListAccessError::Repository { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn log_error_chain(err: &dyn Error) {
    let mut chain = vec![format!("{err}")];
    let mut src = err.source();
    while let Some(e) = src {
        chain.push(format!("{e}"));
        src = e.source();
    }
    log::error!("request failed: {:?}", chain);
}
