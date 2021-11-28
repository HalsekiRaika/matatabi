pub mod authenticate;
pub mod upcoming;
pub mod nf;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    /// FEATURE - Deprecate (next ver)
    #[error("General Error")]
    Error(String)
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::Error(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::web::HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                error: match self {
                    ApiError::Error(..) => "general_error"
                },
                description: &self.to_string()
            }
        )
    }
}