use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::PgPool;

use crate::models::channel::Channel;
use crate::database::models::{channel_object::ChannelObject, Fetch};

use super::{ErrorResponse, ApiError};

pub async fn get_channels(
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Channel>>, ErrorResponse> {
    let ch_all = ChannelObject::fetch_all(&pool).await
        .map_err(|e| ApiError::new(e).report(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter()
        .map(Channel::from)
        .collect::<Vec<_>>();
    Ok(Json(ch_all))
}