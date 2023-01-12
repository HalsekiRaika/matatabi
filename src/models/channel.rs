use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use crate::database::{ChannelId, ChannelObject};

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
    fn from(db_id: ChannelId) -> Self {
        StringId::new(db_id)
    }
}

impl From<ChannelObject> for Channel {
    fn from(obj: ChannelObject) -> Self {
        Self {
            channel_id: StringId::from(obj.channel_id().to_owned()),
            liver_id: obj.liver_id().map(NumId::from),
            logo_url: obj.logo_url().to_owned(),
            published_at: obj.published_at(),
            description: obj.description().to_owned()
        }
    }
}