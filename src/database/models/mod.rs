pub mod id_object;
pub mod affiliation_object;
pub mod livers_object;
pub mod upcoming_object;
pub mod update_signature;
pub mod channel_object;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RawString(String);

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

// FEATURE: There are some differences in the query statements,
//          but most of the processing is common, so I may want to add a default implementation later.
#[async_trait::async_trait]
pub trait Transactable<T> {
    async fn insert(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<T, sqlx::Error>;
    async fn update(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(T, T), sqlx::Error>;
    async fn exists(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
    async fn delete(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<i64, sqlx::Error>;
}


/// Trait used to mediate basic SQL Transactions.
///
/// Use the SQL statement "Returning *" to use the value of the result after the SQL is executed for the return value
#[async_trait::async_trait]
pub trait Transact {
    type TransactItem;

    /// Consume the value and insert it into the database.
    ///
    /// [Ok()]: `T` - Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn insert(self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self::TransactItem, sqlx::Error>;

    /// Consumes a value and deletes the corresponding data from the database.
    ///
    /// [Ok()]: `T` - Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn delete(self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self::TransactItem, sqlx::Error>;

    /// Consumes a value and updates the value in the database.
    ///
    /// [Ok()] `(T, T)` - Return tuple (old, new). Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn update(self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(Self::TransactItem, Self::TransactItem), sqlx::Error>;

    /// Verify that the corresponding data exists in the database.
    ///
    /// Use
    ///
    /// ```sql
    /// SELECT EXISTS(SELECT 1 FROM data WHERE id LIKE $1)
    /// ```
    ///
    /// or otherwise return the Bool value from the database.
    ///
    /// [Ok()]: `bool` - Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn exists(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
}