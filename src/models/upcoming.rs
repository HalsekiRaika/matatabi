use chrono::{DateTime, Utc};
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
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub will_start_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub thumbnail_url: String
}

impl From<FromDatabaseLiveId> for StringId<Live> {
    fn from(database: FromDatabaseLiveId) -> Self {
        StringId::new(database.0)
    }
}

// impl From<FromDatabaseLive> for Live {
//     fn from(database: FromDatabaseLive) -> Self {
//         Self {
//             video_id: database.
//         }
//     }
// }