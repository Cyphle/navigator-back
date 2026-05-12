use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("No username_or_email specified")]
    MissingUsername,

    #[error("Invalid due_date format, expected YYYY-MM-DD")]
    InvalidDateFormat,

    #[error("{message}")]
    Application { message: String, status_code: u16 },
}

impl From<MissingUsernameError> for MiddlewareError {
    fn from(_: MissingUsernameError) -> Self {
        Self::MissingUsername
    }
}

impl From<Box<dyn ApplicationError>> for MiddlewareError {
    fn from(e: Box<dyn ApplicationError>) -> Self {
        Self::Application {
            message: e.get_message(),
            status_code: e.status_code(),
        }
    }
}

impl ResponseError for MiddlewareError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingUsername => StatusCode::UNAUTHORIZED,
            Self::InvalidDateFormat => StatusCode::BAD_REQUEST,
            Self::Application { status_code, .. } => StatusCode::from_u16(*status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        if status.is_server_error() {
            log::error!("middleware error: {}", self);
        }
        let body = match self {
            Self::MissingUsername | Self::InvalidDateFormat => self.to_string(),
            Self::Application { message, .. } => message.clone(),
        };
        HttpResponse::build(status).json(body)
    }
}
