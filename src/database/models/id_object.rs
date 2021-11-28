use serde::{Serialize, Deserialize};
use sqlx::Type;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct AffiliationId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct ChannelId(pub String);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct LiverId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct VideoId(pub String);

use crate::models;

impl From<models::affiliation::AffiliationId> for AffiliationId {
    fn from(id: models::affiliation::AffiliationId) -> Self {
        AffiliationId(id.0)
    }
}

impl From<AffiliationId> for models::affiliation::AffiliationId {
    fn from(id: AffiliationId) -> Self {
        models::affiliation::AffiliationId(id.0)
    }
}

impl From<models::channel::ChannelId> for ChannelId {
    fn from(id: models::channel::ChannelId) -> Self {
        ChannelId(id.0)
    }
}

impl From<ChannelId> for models::channel::ChannelId {
    fn from(id: ChannelId) -> Self {
        models::channel::ChannelId(id.0)
    }
}

impl From<models::liver::LiverId> for LiverId {
    fn from(id: models::liver::LiverId) -> Self {
        LiverId(id.0)
    }
}

impl From<LiverId> for models::liver::LiverId {
    fn from(id: LiverId) -> Self {
        models::liver::LiverId(id.0)
    }
}

impl From<models::upcoming::VideoId> for VideoId {
    fn from(id: models::upcoming::VideoId) -> Self {
        VideoId(id.0)
    }
}

impl From<VideoId> for models::upcoming::VideoId {
    fn from(id: VideoId) -> Self {
        models::upcoming::VideoId(id.0)
    }
}