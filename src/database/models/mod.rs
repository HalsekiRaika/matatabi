pub mod id_object;
pub mod affiliation_object;
pub mod livers_object;
pub mod upcoming_object;
pub mod update_signature;
pub mod channel_object;

/// Trait used to mediate basic SQL Transactions.
///
/// Use the SQL statement "Returning *" to use the value of the result after the SQL is executed for the return value
#[async_trait::async_trait]
pub trait Accessor {
    type Item;

    /// Consume the value and insert it into the database.
    ///
    /// [Ok()]: `T` - Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn insert(self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self::Item, sqlx::Error>;

    /// Consumes a value and deletes the corresponding data from the database.
    ///
    /// [Ok()]: `T` - Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn delete(self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self::Item, sqlx::Error>;

    /// Consumes a value and updates the value in the database.
    ///
    /// [Ok()] `(T, T)` - Return tuple (old, new). Value returned by SQL statement "Returning *".
    ///
    /// [Err()] - Error in sqlx.
    async fn update(self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(Self::Item, Self::Item), sqlx::Error>;

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

    /// Compares its own data with the data present in the database.
    /// The PrimaryKey is the criteria, and the data must exist beforehand.
    ///
    /// [Ok()]: `bool` - Returns true if the Hash of the data in the database matches its own Hash.
    ///
    /// [Err()] - Error in sqlx.
    async fn compare(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait Fetch {
    type Item;
    async fn fetch_all(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Vec<Self::Item>, sqlx::Error>;
}

pub fn hash<T: std::hash::Hash>(hash_obj: &T) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_obj.hash(&mut hasher);
    hasher.finish()
}