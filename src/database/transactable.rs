#[async_trait::async_trait]
pub trait Transactable {
    async fn insert(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(), sqlx::Error>;
    async fn update(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(), sqlx::Error>;
    async fn exists(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
}