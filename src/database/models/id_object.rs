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

use crate::models;

impl From<models::affiliation::AffiliationId> for AffiliationId {
    /// Use to transform webapi model to database model
    fn from(id: models::affiliation::AffiliationId) -> Self {
        AffiliationId(id.0)
    }
}

impl From<AffiliationId> for models::affiliation::AffiliationId {
    /// Use to transform database model to webapi model
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

// Todo: Consider using the following methods to reduce boilerplate
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
pub struct NumId<T> {
    value: i64,
    #[serde(skip)]
    _mark: std::marker::PhantomData<T>
}

#[allow(dead_code)]
impl<T> NumId<T> {
    pub fn new(id: i64) -> NumId<T> {
        Self { value: id, _mark: std::marker::PhantomData }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct StringId<T> {
    value: String,
    #[serde(skip)]
    _mark: std::marker::PhantomData<T>
}

#[allow(dead_code)]
impl<T> StringId<T> {
    pub fn new(id: impl Into<String>) -> StringId<T> {
        Self { value: id.into(), _mark: std::marker::PhantomData }
    }

    pub fn as_ref(&self) -> &str {
        &self.value
    }
}