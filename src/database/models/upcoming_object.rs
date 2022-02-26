use core::option::Option;
use std::str::FromStr;
use chrono::{DateTime, Local};
use crate::database::models::update_signature::UpdateSignature;

use super::id_object::{ChannelId, VideoId};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Lives {
    video_id: VideoId,
    channel_id: Option<ChannelId>,
    title: String,
    description: String,
    published_at: DateTime<Local>,
    updated_at: DateTime<Local>,
    will_start_at: Option<DateTime<Local>>,
    started_at: Option<DateTime<Local>>,
    thumbnail_url: String,
    update_signature: UpdateSignature
}