use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use crate::database::models::id_object::VideoId;
use crate::database::models::upcoming_object::VideoObject;

use super::StringId;
use super::channel::Channel;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub video_id: StringId<Video>,
    pub channel_id: Option<StringId<Channel>>,
    pub title: String,
    pub description: String,
    pub published_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub will_start_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub thumbnail_url: String
}

impl From<VideoId> for StringId<Video> {
    fn from(db_id: VideoId) -> Self {
        StringId::new(db_id)
    }
}

impl From<VideoObject> for Video {
    fn from(database: VideoObject) -> Self {
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