pub mod authenticate;
pub mod upcoming;
pub mod nf;
pub mod affiliation;

pub async fn index() -> actix_web::HttpResponse {
    let data = serde_json::json!({
        "api_name": "matatabi",
        "major_version": dotenv::var("MAJOR_API_VERSION").ok(),
        "minor_version": dotenv::var("MINOR_API_VERSION").ok(),
        "repository": "https://github.com/ReiRokusanami0010/matatabi",
    });
    actix_web::HttpResponse::Ok().json(data)
}

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