use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use crate::database::models::upcoming_object::Lives as FromDatabaseLive;
use crate::database::models::id_object::VideoId as FromDatabaseLiveId;

use super::StringId;
use super::channel::Channel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live {
    pub video_id: StringId<Live>,
    pub channel_id: Option<StringId<Channel>>,
    pub title: String,
    pub description: String,
    pub published_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub will_start_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub thumbnail_url: String
}

impl From<FromDatabaseLiveId> for StringId<Live> {
    fn from(database: FromDatabaseLiveId) -> Self {
        StringId::new(database.0)
    }
}

impl From<FromDatabaseLive> for Live {
    fn from(database: FromDatabaseLive) -> Self {
        let pubs = database.decompose();
        Self {
            video_id: StringId::from(pubs.video_id.clone()),
            channel_id: pubs.channel_id.clone().map(StringId::from),
            title: pubs.title.clone(),
            description: pubs.description.clone(),
            published_at: pubs.published_at,
            updated_at: pubs.updated_at,
            will_start_at: pubs.will_start_at,
            started_at: pubs.started_at,
            thumbnail_url: pubs.thumbnail_url
        }
    }
}