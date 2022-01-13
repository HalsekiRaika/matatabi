pub mod id_object;
pub mod affiliation_object;
pub mod livers_object;
pub mod upcoming_object;
pub mod update_signature;

pub trait Printable {
    fn get_primary_name(&self) -> String;
    fn get_secondary_name(&self) -> String;
}

#[async_trait::async_trait]
pub trait Updatable {
    fn apply_signature(&self, sign: i64) -> Self;
    fn is_empty_sign(&self) -> bool;
    fn get_signature(&self) -> i64;
    async fn can_update(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait Transactable<T> {
    async fn insert(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<T, sqlx::Error>;
    async fn update(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(T, T), sqlx::Error>;
    async fn exists(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
    async fn delete(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<i64, sqlx::Error>;
}