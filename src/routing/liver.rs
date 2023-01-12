use std::collections::HashMap;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::http::StatusCode;
use sqlx::PgPool;
use crate::models::Liver;
use crate::database::LiverObject;
use super::{ApiError, ErrorResponse};

pub async fn get_livers(
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Liver>>, ErrorResponse> {
    let liver_all = LiverObject::fetch_all(&pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter()
        .map(Liver::from)
        .collect();
    Ok(Json(liver_all))
}

pub async fn get_livers_filtered(
    Query(affiliation_id): Query<HashMap<String, u64>>,
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Liver>>, ErrorResponse> {
    let id = affiliation_id.into_iter().find(|(param_key, _)| param_key == "affiliated")
        .map(|(_, id)| id)
        .unwrap_or(0);
    let livers = LiverObject::fetch_filtered_affiliation(id as i64, &pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter()
        .map(Liver::from)
        .collect();
    Ok(Json(livers))
}