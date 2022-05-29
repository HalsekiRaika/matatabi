#![allow(dead_code)]

use core::option::Option;
use chrono::{DateTime, Local};
use sqlx::{Row, Postgres, Transaction};
use crate::database::models::{Printable, Updatable, Transactable};
use crate::database::models::update_signature::UpdateSignature;

use super::id_object::{ChannelId, VideoId};

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

impl Printable for Lives {
    fn get_primary_name(&self) -> String {
        self.video_id.clone().0
    }

    fn get_secondary_name(&self) -> String {
        self.channel_id.clone().unwrap_or_else(|| ChannelId("none".to_string())).0
    }
}

#[async_trait::async_trait]
impl Updatable for Lives {
    fn apply_signature(&self, sign: i64) -> Self {
        let mut a = self.clone();
        a.update_signatures = UpdateSignature(sign);
        a
    }

    fn is_empty_sign(&self) -> bool {
        self.update_signatures.0 <= 1
    }

    fn get_signature(&self) -> i64 {
        self.update_signatures.0
    }

    async fn can_update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let may_older: i64 = sqlx::query(r#"
            SELECT update_signatures FROM lives WHERE video_id LIKE $1
        "#).bind(&self.video_id)
            .fetch_one(&mut *transaction)
            .await?
            .get::<i64, _>(0);
        Ok(self.update_signatures.0 > may_older)
    }
}

#[async_trait::async_trait]
impl Transactable<Lives> for Lives {
    async fn insert(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, sqlx::Error> {
        // language=SQL
        let insert = sqlx::query_as::<_, Lives>(r#"
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

    async fn update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self, Self), sqlx::Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Lives>(r#"
            SELECT * FROM lives WHERE video_id LIKE $1
        "#).bind(&self.video_id)
           .fetch_one(&mut *transaction)
           .await?;
        // language=SQL
        let new = sqlx::query_as::<_, Lives>(r#"
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
           .get::<bool, _>(0);

        Ok(video_exists)
    }


    async fn delete(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<i64, sqlx::Error> {
        // language=SQL
        let delete = sqlx::query_as::<_, UpdateSignature>(r#"
            DELETE FROM lives WHERE video_id LIKE $1 RETURNING update_signatures
        "#).bind(&self.video_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(delete.0)
    }
}

pub struct LivesBuilder {
    pub video_id: VideoId,
    pub channel_id: Option<ChannelId>,
    pub title: String,
    pub description: String,
    pub published_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub will_start_at: Option<DateTime<Local>>,
    pub started_at: Option<DateTime<Local>>,
    pub thumbnail_url: String,
    pub update_signature: UpdateSignature,
    #[doc(hidden)]
    pub init: ()
}

impl Default for LivesBuilder {
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
            update_signature: UpdateSignature::default(),
            init: ()
        }
    }
}

impl LivesBuilder {
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
            update_signatures: self.update_signature
        }
    }
}