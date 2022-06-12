use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type, FromRow)]
#[sqlx(transparent)]
pub struct AffiliationId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, FromRow)]
#[sqlx(transparent)]
pub struct ChannelId(pub String);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Type, FromRow)]
#[sqlx(transparent)]
pub struct LiverId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, FromRow)]
#[sqlx(transparent)]
pub struct VideoId(pub String);

impl Display for AffiliationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "affiliation_id: {}", self.0)
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel_id: {}", self.0)
    }
}

impl Display for LiverId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "liver_id: {}", self.0)
    }
}

impl Display for VideoId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "video_id: {}", self.0)
    }
}