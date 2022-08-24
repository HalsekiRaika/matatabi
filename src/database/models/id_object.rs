use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, FromRow, Default)]
#[sqlx(transparent)]
pub struct AffiliationId(i64);

impl AffiliationId {
    pub fn new(id: impl Into<i64>) -> Self {
        Self(id.into())
    }
}

impl From<i64> for AffiliationId {
    fn from(prime: i64) -> Self {
        AffiliationId(prime)
    }
}

impl From<AffiliationId> for i64 {
    fn from(inner: AffiliationId) -> Self {
        inner.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, FromRow, Default)]
#[sqlx(transparent)]
pub struct LiverId(i64);

impl LiverId {
    pub fn new(id: impl Into<i64>) -> Self {
        Self(id.into())
    }
}

impl From<i64> for LiverId {
    fn from(prime: i64) -> Self {
        LiverId(prime)
    }
}

impl From<LiverId> for i64 {
    fn from(inner: LiverId) -> Self {
        inner.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, FromRow, Default)]
#[sqlx(transparent)]
pub struct ChannelId(String);

impl ChannelId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<String> for ChannelId {
    fn from(prime: String) -> Self {
        ChannelId(prime)
    }
}

impl From<ChannelId> for String {
    fn from(inner: ChannelId) -> Self {
        inner.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, FromRow, Default)]
#[sqlx(transparent)]
pub struct VideoId(String);

impl VideoId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<String> for VideoId {
    fn from(prime: String) -> Self {
        VideoId(prime)
    }
}

impl From<VideoId> for String {
    fn from(inner: VideoId) -> Self {
        inner.0
    }
}

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