use axum::Json;
use serde_json::Value;

pub async fn version() -> Json<Value> {
    Json(serde_json::json!({
        "api_name": "matatabi",
        "major_version": dotenv::var("MAJOR_API_VERSION").ok(),
        "minor_version": dotenv::var("MINOR_API_VERSION").ok(),
        "repository": "https://github.com/ReiRokusanami0010/matatabi",
    }))
}
