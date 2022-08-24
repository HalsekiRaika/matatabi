#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use chrono::{DateTime, Local};
use sqlx::{Row, Postgres, Transaction};

use super::{Accessor, hash, Fetch};
use super::id_object::{ChannelId, VideoId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::FromRow)]
pub struct VideoObject {
    video_id: VideoId,
    channel_id: Option<ChannelId>,
    title: String,
    description: String,
    published_at: Option<DateTime<Local>>,
    updated_at: Option<DateTime<Local>>,
    will_start_at: Option<DateTime<Local>>,
    started_at: Option<DateTime<Local>>,
    thumbnail_url: String
}

impl VideoObject {
    pub fn video_id(&self) -> &VideoId {
        &self.video_id
    }

    pub fn channel_id(&self) -> Option<&ChannelId> {
        self.channel_id.as_ref()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn published_at(&self) -> Option<DateTime<Local>> {
        self.published_at
    }

    pub fn updated_at(&self) -> Option<DateTime<Local>> {
        self.updated_at
    }

    pub fn will_start_at(&self) -> Option<DateTime<Local>> {
        self.will_start_at
    }

    pub fn started_at(&self) -> Option<DateTime<Local>> {
        self.started_at
    }

    pub fn thumbnail_url(&self) -> &str {
        &self.thumbnail_url
    }
}

impl Display for VideoObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "live(video) >> {}, title: {}", self.video_id, self.title)
    }
}

#[async_trait::async_trait]
impl Accessor for VideoObject {
    type Item = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, sqlx::Error> {
        // language=SQL
        let insert = sqlx::query_as::<_, Self>(r#"
            INSERT INTO videos
                (video_id, channel_id, title, description,
                published_at, updated_at, will_start_at, started_at,
                thumbnail_url)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#).bind(&self.video_id)
           .bind(&self.channel_id)
           .bind(&self.title)
           .bind(&self.description)
           .bind(self.published_at)
           .bind(self.updated_at)
           .bind(self.will_start_at)
           .bind(self.started_at)
           .bind(&self.thumbnail_url)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(insert)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, sqlx::Error> {
        // language=SQL
        let delete = sqlx::query_as::<_, Self>(r#"
            DELETE FROM videos WHERE video_id LIKE $1 RETURNING *
        "#).bind(&self.video_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(delete)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self::Item, Self::Item), sqlx::Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM videos WHERE video_id LIKE $1
        "#).bind(&self.video_id)
            .fetch_one(&mut *transaction)
            .await?;
        // language=SQL
        let new = sqlx::query_as::<_, Self>(r#"
            UPDATE videos SET title = $1, description = $2, updated_at = $3, will_start_at = $4, started_at = $5 WHERE video_id LIKE $6
            RETURNING *
        "#).bind(&self.title)
           .bind(&self.description)
           .bind(self.updated_at)
           .bind(self.will_start_at)
           .bind(self.started_at)
           .bind(&self.video_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok((old, new))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let video_exists = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM videos WHERE video_id LIKE $1)
        "#).bind(&self.video_id)
            .fetch_one(&mut *transaction)
            .await?
            .try_get::<bool, _>(0)?;
        Ok(video_exists)
    }

    async fn compare(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error> {
        let com = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM videos WHERE video_id = $1
        "#).bind(&self.channel_id)
           .fetch_optional(&mut *transaction)
           .await?;
        
        let com = if let Some(db) = com {
            let db = hash(&db);
            let my = hash(&self);
            db == my
        } else { false };
        Ok(com)
    }
}

#[async_trait::async_trait]
impl Fetch for VideoObject {
    type Item = Self;
    async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM videos
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

pub struct InitVideoObject {
    pub video_id: VideoId,
    pub channel_id: Option<ChannelId>,
    pub title: String,
    pub description: String,
    pub published_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub will_start_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub thumbnail_url: String,
    #[doc(hidden)]
    pub init: ()
}

impl Default for InitVideoObject {
    fn default() -> Self {
        Self {
            video_id: VideoId::default(),
            channel_id: None,
            title: "none".to_string(),
            description: "none".to_string(),
            published_at: None,
            updated_at: None,
            will_start_at: None,
            started_at: None,
            thumbnail_url: "none".to_string(),
            init: ()
        }
    }
}

impl InitVideoObject {
    pub fn build(self) -> VideoObject {
        VideoObject {
            video_id: self.video_id,
            channel_id: self.channel_id,
            title: self.title,
            description: self.description,
            published_at: self.published_at,
            updated_at: self.updated_at,
            will_start_at: self.will_start_at,
            started_at: self.started_at,
            thumbnail_url: self.thumbnail_url,
        }
    }
}

impl VideoObject {
    pub fn decompose(self) -> InitVideoObject {
        InitVideoObject {
            video_id: self.video_id,
            channel_id: self.channel_id,
            title: self.title,
            description: self.description,
            published_at: self.published_at,
            updated_at: self.updated_at,
            will_start_at: self.will_start_at,
            started_at: self.started_at,
            thumbnail_url: self.thumbnail_url,
            init: ()
        }
    }
}