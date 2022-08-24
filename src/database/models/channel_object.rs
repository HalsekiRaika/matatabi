#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use chrono::{DateTime, Local};
use sqlx::{Row, Postgres, Transaction, Error};

use super::{Accessor, hash, Fetch};
use super::id_object::{ChannelId, LiverId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::FromRow)]
pub struct ChannelObject {
    channel_id: ChannelId,
    liver_id: Option<LiverId>,
    logo_url: String,
    published_at: DateTime<Local>,
    description: String
}

impl Display for ChannelObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel >> {}, liver(id): {:?}", self.channel_id, self.liver_id)
    }
}

impl ChannelObject {
    pub fn published_at(&self) -> DateTime<Local> {
        self.published_at
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id
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
impl Accessor for ChannelObject {
    type Item = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let ins = sqlx::query_as::<_, Self>(r#"
            INSERT INTO channels (channel_id, liver_id, logo_url, published_at, description)
             VALUES ($1, $2, $3, $4, $5)
            RETURNING *
        "#).bind(&self.channel_id)
           .bind(self.liver_id)
           .bind(&self.logo_url)
           .bind(self.published_at)
           .bind(&self.description)
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
            UPDATE channels SET description = $1 WHERE channel_id LIKE $2
            RETURNING *
        "#).bind(&self.description)
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

    async fn compare(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error> {
        let com = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM channels WHERE channel_id LIKE $1
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
impl Fetch for ChannelObject {
    type Item = Self;
    async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM channels
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

pub struct ChannelObjectBuilder {
    pub channel_id: ChannelId,
    pub liver_id: Option<LiverId>,
    pub logo_url: String,
    pub published_at: DateTime<Local>,
    pub description: String,
    #[doc(hidden)]
    pub init: ()
}

impl Default for ChannelObjectBuilder {
    fn default() -> Self {
        Self {
            channel_id: ChannelId::default(),
            liver_id: None,
            logo_url: "none".to_string(),
            published_at: Local::now(),
            description: "none".to_string(),
            init: ()
        }
    }
}

impl ChannelObjectBuilder {
    pub fn build(self) -> ChannelObject {
        ChannelObject {
            channel_id: self.channel_id,
            liver_id: self.liver_id,
            logo_url: self.logo_url,
            published_at: self.published_at,
            description: self.description,
        }
    }
}