use std::future::Future;
use std::pin::Pin;
use chrono::{DateTime, Local};
use sqlx::{Error, Row, Postgres, Transaction};
use crate::database::models::id_object::{ChannelId, LiverId};
use crate::database::models::{Printable, Transactable, Updatable};
use crate::database::models::update_signature::UpdateSignature;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Channels {
    channel_id: ChannelId,
    liver_id: Option<LiverId>,
    logo_url: String,
    published_at: DateTime<Local>,
    description: String,
    update_signatures: UpdateSignature
}

impl Channels {
    
}

impl Printable for Channels {
    fn get_primary_name(&self) -> String {
        self.channel_id.clone().0
    }

    fn get_secondary_name(&self) -> String {
        self.liver_id.unwrap_or(LiverId(0)).0.to_string()
    }
}

#[async_trait::async_trait]
impl Updatable for Channels {
    fn apply_signature(&self, sign: i64) -> Self {
        let mut a = self.clone();
        a.update_signatures = UpdateSignature(sign);
        a
    }

    fn is_empty_sign(&self) -> bool {
        self.update_signatures.0 <= 1
    }

    fn get_signature(&self) -> i64 {
        self.update_signatures.0.clone()
    }

    async fn can_update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let may_older: i64 = sqlx::query(r#"
            SELECT update_signatures FROM channels WHERE channel_id = $1
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?
           .get::<i64, _>(0);
        Ok(self.update_signatures.0 > may_older)
    }
}

#[async_trait::async_trait]
impl Transactable<Channels> for Channels {
    async fn insert(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, sqlx::Error> {
        // language=SQL
        let insert = sqlx::query_as::<_, Channels>(r#"
            INSERT INTO channels (channel_id, liver_id, logo_url, published_at, description, update_signatures)
             VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#).bind(&self.channel_id)
           .bind(self.liver_id)
           .bind(&self.logo_url)
           .bind(self.published_at.naive_local())
           .bind(&self.description)
           .bind(self.update_signatures)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(insert)
    }

    async fn update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self, Self), sqlx::Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Channels>(r#"
            SELECT * FROM channels WHERE channel_id = $1
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?;
        // language=SQL
        let new = sqlx::query_as::<_, Channels>(r#"
            UPDATE channels
              SET description = $1, update_signatures = $2
            WHERE channel_id = $3
            RETURNING *
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok((old, new))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let channel_exists = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM channels WHERE channel_id LIKE $1)
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?
           .get::<bool, _>(0);

        Ok(channel_exists)
    }

    async fn delete(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<i64, sqlx::Error> {
        // language=SQL
        let del = sqlx::query_as::<_, LiverId>(r#"
            DELETE FROM channels WHERE channel_id = $1 RETURNING liver_id
        "#).bind(&self.channel_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del.0)
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