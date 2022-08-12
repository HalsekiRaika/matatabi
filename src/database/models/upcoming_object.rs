#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use chrono::{DateTime, Local};
use sqlx::{Row, Postgres, Transaction};

use super::Transact;
use super::id_object::{ChannelId, VideoId};
use super::update_signature::{UpdateSignature, LatestEq, Signed, Version};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Lives {
    video_id: VideoId,
    channel_id: Option<ChannelId>,
    title: String,
    description: String,
    published_at: Option<DateTime<Local>>,
    updated_at: Option<DateTime<Local>>,
    will_start_at: Option<DateTime<Local>>,
    started_at: Option<DateTime<Local>>,
    thumbnail_url: String,
    update_signatures: UpdateSignature
}

impl Lives {
    pub async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM lives
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

impl Display for Lives {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "live(video) >> {}, title: {}", self.video_id, self.title)
    }
}

impl Version for Lives {
    fn version(&self) -> UpdateSignature {
        self.update_signatures
    }
}

#[async_trait::async_trait]
impl Signed for Lives {
    async fn sign(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<UpdateSignature, sqlx::Error> {
        // language=SQL
        let current = sqlx::query(r#"
            SELECT update_signatures FROM lives WHERE video_id LIKE $1
        "#).bind(&self.video_id)
            .fetch_one(&mut *transaction)
            .await?
            .try_get::<UpdateSignature, _>(0)?;
        Ok(current)
    }
}

impl LatestEq for Lives {
    type ComparisonItem = Self;

    fn apply(self, sign: UpdateSignature) -> Self::ComparisonItem {
        let mut a = self;
        a.update_signatures = sign;
        a
    }

    fn version_compare(&self, compare: UpdateSignature) -> bool {
        self.update_signatures.0 > compare.0
    }

    fn irregular_sign(&self) -> bool {
        self.update_signatures.0 <= 1
    }
}

#[async_trait::async_trait]
impl Transact for Lives {
    type TransactItem = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::TransactItem, sqlx::Error> {
        // language=SQL
        let insert = sqlx::query_as::<_, Self>(r#"
            INSERT INTO lives
                (video_id, channel_id, title, description,
                published_at, updated_at, will_start_at, started_at,
                thumbnail_url, update_signatures)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
           .bind(self.update_signatures)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(insert)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::TransactItem, sqlx::Error> {
        // language=SQL
        let delete = sqlx::query_as::<_, Self>(r#"
            DELETE FROM lives WHERE video_id LIKE $1 RETURNING *
        "#).bind(&self.video_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(delete)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self::TransactItem, Self::TransactItem), sqlx::Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM lives WHERE video_id LIKE $1
        "#).bind(&self.video_id)
            .fetch_one(&mut *transaction)
            .await?;
        // language=SQL
        let new = sqlx::query_as::<_, Self>(r#"
            UPDATE lives
            SET title = $1, description = $2, updated_at = $3,
                will_start_at = $4, started_at = $5, update_signatures = $6
            WHERE video_id LIKE $7
            RETURNING *
        "#).bind(&self.title)
           .bind(&self.description)
           .bind(self.updated_at)
           .bind(self.will_start_at)
           .bind(self.started_at)
           .bind(self.update_signatures)
           .bind(&self.video_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok((old, new))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let video_exists = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM lives WHERE video_id LIKE $1)
        "#).bind(&self.video_id)
            .fetch_one(&mut *transaction)
            .await?
            .try_get::<bool, _>(0)?;
        Ok(video_exists)
    }
}

pub struct InitLives {
    pub video_id: VideoId,
    pub channel_id: Option<ChannelId>,
    pub title: String,
    pub description: String,
    pub published_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub will_start_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub thumbnail_url: String,
    pub update_signatures: UpdateSignature,
    #[doc(hidden)]
    pub init: ()
}

impl Default for InitLives {
    fn default() -> Self {
        Self {
            video_id: VideoId("none".to_string()),
            channel_id: None,
            title: "none".to_string(),
            description: "none".to_string(),
            published_at: None,
            updated_at: None,
            will_start_at: None,
            started_at: None,
            thumbnail_url: "none".to_string(),
            update_signatures: UpdateSignature::default(),
            init: ()
        }
    }
}

impl InitLives {
    pub fn build(self) -> Lives {
        Lives {
            video_id: self.video_id,
            channel_id: self.channel_id,
            title: self.title,
            description: self.description,
            published_at: self.published_at,
            updated_at: self.updated_at,
            will_start_at: self.will_start_at,
            started_at: self.started_at,
            thumbnail_url: self.thumbnail_url,
            update_signatures: self.update_signatures
        }
    }
}

impl Lives {
    pub fn decompose(self) -> InitLives {
        InitLives {
            video_id: self.video_id,
            channel_id: self.channel_id,
            title: self.title,
            description: self.description,
            published_at: self.published_at,
            updated_at: self.updated_at,
            will_start_at: self.will_start_at,
            started_at: self.started_at,
            thumbnail_url: self.thumbnail_url,
            update_signatures: self.update_signatures,
            init: ()
        }
    }
}