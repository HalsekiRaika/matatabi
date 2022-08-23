use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use sqlx::PgPool;
use crate::models::affiliation::Affiliation;
use crate::database::models::affiliation_object::AffiliationObject;

use super::ApiError;
use super::ErrorResponse;

pub async fn get_affiliations(
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Affiliation>>, ErrorResponse> {
    let aff_all = AffiliationObject::fetch_all(&pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter()
        .map(Affiliation::from)
        .collect();
    Ok(Json(aff_all))
}

pub async fn get_affiliation_from_id(
    Path(id): Path<u64>,
    Extension(pool): Extension<PgPool>
) -> Result<Json<Affiliation>, ErrorResponse> {
    let aff = AffiliationObject::fetch_name_from_id(id as i64, &pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .map(Affiliation::from)
        .ok_or_else(|| ApiError::reason(format!("{} is not found.", id)).report(StatusCode::NOT_FOUND))?;
    Ok(Json(aff))
}

