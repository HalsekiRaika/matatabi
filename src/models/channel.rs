use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::liver::LiverId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub channel_id: ChannelId,
    pub liver_id: LiverId,
    pub logo_url: String,
    pub published_at: DateTime<Utc>,
    pub description: String
}