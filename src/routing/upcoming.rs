use axum::{Extension, Json};
use axum::http::StatusCode;
use sqlx::PgPool;
use crate::models::Video;
use crate::database::{Fetch, VideoObject};

use super::ApiError;
use super::ErrorResponse;

pub async fn get_upcomings(
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Video>>, ErrorResponse> {
    let live_all = VideoObject::fetch_all(&pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter()
        .map(Video::from)
        .collect::<Vec<_>>();
    Ok(Json(live_all))
}