use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use super::channel::ChannelId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingLive {
    pub video_id: VideoId,
    pub channel_id: ChannelId,
    pub title: String,
    pub description: String,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub will_start_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub thumbnail_url: String
}