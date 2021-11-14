use actix_web::{ResponseError, HttpResponse};
use actix_web::http::StatusCode;
use thiserror::Error;

use crate::models::error::ApiError;



#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Already exist credentials")]
    AlreadyExistCredentialsError,
    #[error("Invalid Authentication credentials")]
    InvalidCredentialsError,
    #[error("Failed connect Authentication Server")]
    FailedConnectionAuthServer
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::AlreadyExistCredentialsError => StatusCode::CONFLICT,
            AuthError::InvalidCredentialsError => StatusCode::UNAUTHORIZED,
            AuthError::FailedConnectionAuthServer => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                AuthError::AlreadyExistCredentialsError => "already_exist_account",
                AuthError::InvalidCredentialsError => "invalid_auth_credentials",
                AuthError::FailedConnectionAuthServer => "failed_connect_auth_server"
            },
            description: &self.to_string()
        })
    }
}