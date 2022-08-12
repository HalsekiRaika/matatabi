use axum::{Extension, Json};
use axum::http::StatusCode;
use sqlx::PgPool;
use crate::models::upcoming::Live;
use crate::database::models::upcoming_object::Lives as UpcomingObject;

use super::ApiError;
use super::ErrorResponse;

pub async fn get_upcomings(
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Live>>, ErrorResponse> {
    let live_all = UpcomingObject::fetch_all(&pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter()
        .map(Live::from)
        .collect::<Vec<_>>();
    Ok(Json(live_all))
}