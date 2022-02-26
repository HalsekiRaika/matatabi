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

pub struct LivesBuilder {
    pub video_id: VideoId,
    pub channel_id: Option<ChannelId>,
    pub title: String,
    pub description: String,
    pub published_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub will_start_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub thumbnail_url: String,
    pub update_signature: UpdateSignature,
    #[doc(hidden)]
    pub init: ()
}

impl Default for LivesBuilder {
    fn default() -> Self {
        Self {
            video_id: VideoId("none".to_string()),
            channel_id: None,
            title: "none".to_string(),
            description: "none".to_string(),
            published_at: Local::now(),
            updated_at: Local::now(),
            will_start_at: None,
            started_at: None,
            thumbnail_url: "none".to_string(),
            update_signature: UpdateSignature::default(),
            init: ()
        }
    }
}

impl LivesBuilder {
    pub fn build(self) -> Lives {
        Lives {
            video_id: self.video_id,
            channel_id: self.channel_id,
            title: self.title,
            description: self.description,
            published_at: self.published_at,
            updated_at: self.updated_at,
            will_start_at: self.will_start_at,
            started_at: self.started_at,
            thumbnail_url: self.thumbnail_url,
            update_signature: self.update_signature
        }
    }
}