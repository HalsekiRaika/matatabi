use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use crate::database::models::id_object::ChannelId;
use crate::database::models::channel_object::Channels as FromDatabaseChannel;

use super::{NumId, StringId};
use super::liver::Liver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub channel_id: StringId<Channel>,
    pub liver_id: Option<NumId<Liver>>,
    pub logo_url: String,
    pub published_at: DateTime<Local>,
    pub description: String
}

impl From<ChannelId> for StringId<Channel> {
    fn from(database: ChannelId) -> Self {
        StringId::new(database.0)
    }
}

impl From<FromDatabaseChannel> for Channel {
    fn from(database: FromDatabaseChannel) -> Self {
        Self {
            channel_id: StringId::from(database.breach_channel_id()),
            liver_id: database.liver_id().map(NumId::from),
            logo_url: database.breach_logo_url(),
            published_at: database.published_at(),
            description: database.breach_description()
        }
    }
}