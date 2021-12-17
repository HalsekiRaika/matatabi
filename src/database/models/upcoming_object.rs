use core::option::Option;
use std::str::FromStr;
use chrono::NaiveDateTime;
use crate::database::models::update_signature::UpdateSignature;

use super::id_object::{ChannelId, VideoId};

#[derive(Debug, Clone)]
pub struct Lives {
    pub video_id: VideoId,
    pub channel_id: ChannelId,
    pub title: String,
    pub description: String,
    pub published_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub will_start_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub thumbnail_url: String,
    pub update_signature: UpdateSignature
}

impl Lives {
    pub fn new(
        video_id: impl Into<String>, channel_id: impl Into<String>,
        title: impl Into<String>, description: impl Into<String>,
        published_at: impl Into<String>, updated_at: impl Into<String>,
        will_start_at: Option<String>, started_at: Option<String>,
        thumbnail_url: impl Into<String>
      ) -> Self {
        let published_at= NaiveDateTime::from_str(&published_at.into()).unwrap();
        let updated_at = NaiveDateTime::from_str(&updated_at.into()).unwrap();
        let will_start_at = if let Some(will_start_at) = will_start_at {
            Some(NaiveDateTime::from_str(&will_start_at).unwrap())
        } else { None };
        let started_at = if let Some(started_at) = started_at {
            Some(NaiveDateTime::from_str(&started_at).unwrap())
        } else { None };
        let update_sign = UpdateSignature::default();
        Self {
            video_id: VideoId(video_id.into()),
            channel_id: ChannelId(channel_id.into()),
            title: title.into(),
            description: description.into(),
            published_at, updated_at,
            will_start_at, started_at,
            thumbnail_url: thumbnail_url.into(),
            update_signature: update_sign
        }
    }

    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO lives (
                video_id, channel_id,
                title, description,
                published_at, updated_at,
                will_start_at, started_at,
                thumbnail_url, update_signatures
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
            ",
            self.video_id.clone() as VideoId,
            self.channel_id.clone() as ChannelId,
            &self.title,
            &self.description,
            self.published_at,
            self.updated_at,
            self.will_start_at,
            self.started_at,
            &self.thumbnail_url,
            self.update_signature as UpdateSignature
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}