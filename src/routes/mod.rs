pub mod upcoming;
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

pub fn routing(serv_conf: &mut actix_web::web::ServiceConfig) {
    serv_conf.service(
        actix_web::web::scope("/v0")
            .configure(affiliation_route)
    );
}

fn affiliation_route(serv_conf: &mut actix_web::web::ServiceConfig) {
    serv_conf.service(affiliation::get_affiliations);

    serv_conf.service(
        actix_web::web::scope("affiliations")
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    /// FEATURE - Deprecate (next ver)
    #[error("General Error")]
    Error(String),
    #[error("Internal Server Error from Database: {0}")]
    SqlxDatabaseError(#[from] sqlx::Error)
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::Error(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SqlxDatabaseError(..) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                error: match self {
                    ApiError::Error(..) => "general_error",
                    ApiError::SqlxDatabaseError(..) => "database_error"
                },
                description: &self.to_string()
            }
        )
    }
}