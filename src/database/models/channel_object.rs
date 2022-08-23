#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use chrono::{DateTime, Local};
use sqlx::{Row, Postgres, Transaction, Error};

use super::Accessor;
use super::id_object::{ChannelId, LiverId};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Channels {
    channel_id: ChannelId,
    liver_id: Option<LiverId>,
    logo_url: String,
    published_at: DateTime<Local>,
    description: String
}

impl Display for Channels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel >> {}, liver(id): {:?}", self.channel_id, self.liver_id)
    }
}

impl Channels {
    pub fn published_at(&self) -> DateTime<Local> {
        self.published_at
    }

    pub fn breach_channel_id(&self) -> ChannelId {
        self.channel_id.clone()
    }

    pub fn liver_id(&self) -> Option<LiverId> {
        self.liver_id
    }

    pub fn logo_url(&self) -> &str {
        &self.logo_url
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[async_trait::async_trait]
impl Accessor for Channels {
    type Item = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let ins = sqlx::query_as::<_, Self>(r#"
            INSERT INTO channels (channel_id, liver_id, logo_url, published_at, description, update_signatures)
             VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#).bind(&self.channel_id)
           .bind(self.liver_id)
           .bind(&self.logo_url)
           .bind(self.published_at)
           .bind(&self.description)
           .bind(self.update_signatures)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(ins)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let del = sqlx::query_as::<_, Self>(r#"
            DELETE FROM channels WHERE channel_id LIKE $1 RETURNING *
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self::Item, Self::Item), Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM channels WHERE channel_id LIKE $1
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?;
        // language=SQL
        let new = sqlx::query_as::<_, Self>(r#"
            UPDATE channels
              SET description = $1, update_signatures = $2
            WHERE channel_id LIKE $3
            RETURNING *
        "#).bind(&self.description)
           .bind(self.update_signatures)
           .bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok((old, new))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, Error> {
        // language=SQL
        let channel_exists = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM channels WHERE channel_id LIKE $1)
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?
           .try_get::<bool, _>(0)?;

        Ok(channel_exists)
    }
}

pub struct ChannelsBuilder {
    pub channel_id: ChannelId,
    pub liver_id: Option<LiverId>,
    pub logo_url: String,
    pub published_at: DateTime<Local>,
    pub description: String,
    pub update_signatures: UpdateSignature,
    #[doc(hidden)]
    pub init: ()
}

impl Default for ChannelsBuilder {
    fn default() -> Self {
        Self {
            channel_id: ChannelId("none".to_string()),
            liver_id: None,
            logo_url: "none".to_string(),
            published_at: Local::now(),
            description: "none".to_string(),
            update_signatures: UpdateSignature::default(),
            init: ()
        }
    }
}

impl ChannelsBuilder {
    pub fn build(self) -> Channels {
        Channels {
            channel_id: self.channel_id,
            liver_id: self.liver_id,
            logo_url: self.logo_url,
            published_at: self.published_at,
            description: self.description,
            update_signatures: self.update_signatures
        }
    }
}