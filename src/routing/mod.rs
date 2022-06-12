pub mod affiliation;

use axum::http::StatusCode;
use axum::Json;
use serde::{Serialize, Deserialize};
use serde_json::Value;

type ErrorResponse = (StatusCode, Json<ApiError>);

pub async fn version() -> Json<Value> {
    Json(serde_json::json!({
        "api_name": "matatabi",
        "major_version": dotenv::var("MAJOR_API_VERSION").ok(),
        "minor_version": dotenv::var("MINOR_API_VERSION").ok(),
        "repository": "https://github.com/ReiRokusanami0010/matatabi",
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    reason: String
}

impl ApiError {
    pub fn new<E: std::error::Error>(error: E) -> ApiError {
        Self { reason: error.to_string() }
    }

    pub fn reason<I: Into<String>>(reason: I) -> ApiError {
        Self { reason: reason.into() }
    }

    pub fn report(self, status_code: StatusCode) -> (StatusCode, Json<Self>) {
        (status_code, Json(self))
    }
}