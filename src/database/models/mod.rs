pub mod affiliation_object;
pub mod id_object;
pub mod upcoming_object;
pub mod update_signature;

pub trait Printable {
    fn get_primary_name(&self) -> String;
}

#[async_trait::async_trait]
pub trait Updatable {
    fn apply_signature(&self, sign: i64) -> Self;
    fn isEmptySign(&self) -> bool;
    async fn can_update(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
}