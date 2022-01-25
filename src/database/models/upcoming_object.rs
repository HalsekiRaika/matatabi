use core::option::Option;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use crate::database::models::update_signature::UpdateSignature;

use super::id_object::{ChannelId, VideoId};

#[derive(Debug, Clone)]
pub struct Lives {
    video_id: VideoId,
    channel_id: Option<ChannelId>,
    title: String,
    description: String,
    published_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    will_start_at: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    thumbnail_url: String,
    update_signature: UpdateSignature
}