use chrono::NaiveDateTime;

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
    pub thumbnail_url: String
}

impl Lives {
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
                thumbnail_url
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9
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
            self.thumbnail_url
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}