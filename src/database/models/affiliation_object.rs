use super::id_object::AffiliationId;
use super::update_signature::UpdateSignature;
use sqlx::{FromRow, Row, Transaction};
use sqlx::postgres::Postgres;
use crate::database::models::{Printable, Updatable, Transactable, RawString};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Affiliations {
    affiliation_id: AffiliationId,
    name: String,
    update_signatures: UpdateSignature
}

#[derive(Debug, Clone, FromRow)]
struct RawAffiliations {
    affiliation_id: Option<i64>,
    name: Option<String>,
    update_signatures: Option<i64>
}

impl From<RawAffiliations> for Affiliations {
    fn from(raw: RawAffiliations) -> Self {
        let id = if let Some(id) = raw.affiliation_id { id } else { 0 };
        let name = if let Some(name) = raw.name { name } else { "none".to_string() };
        let sign = if let Some(sign) = raw.update_signatures { sign } else { 0 };

        Affiliations::new(id, name, sign)
    }
}

impl Affiliations {
    pub fn new(id: i64, name: impl Into<String>, update_signatures: i64) -> Affiliations {
        Self { affiliation_id: AffiliationId(id), name: name.into(), update_signatures: UpdateSignature(update_signatures) }
    }

    pub fn get_affiliation_id(&self) -> AffiliationId { self.affiliation_id }
    pub fn get_name(&self) -> &str { &self.name }
}

/// The reason for using sqlx::Executor<'a, Database = Postgres>
/// is that it is an abstraction to support the types provided by the actix data propagation.
impl Affiliations {
    pub async fn fetch_id_from_name<'a, E>(
        name: impl Into<String>,
        transaction: E
    ) -> Result<Option<AffiliationId>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let id = sqlx::query_as::<_, AffiliationId>(r#"
            SELECT affiliation_id FROM affiliations WHERE name LIKE $1
        "#).bind(name.into())
           .fetch_optional(transaction)
           .await?;
        Ok(id)
    }

    pub async fn fetch_name_from_id<'a, E>(
        id: AffiliationId,
        transaction: E
    ) -> Result<Option<String>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy{
        // language=SQL
        let name = sqlx::query_as::<_, RawString>(r#"
            SELECT name FROM affiliations WHERE affiliation_id = $1
        "#).bind(id.0)
           .fetch_optional(transaction)
           .await?;

        Ok(Some(name.unwrap().0))
    }

    pub async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

impl Printable for Affiliations {
    fn get_primary_name(&self) -> String {
        self.name.clone()
    }
    fn get_secondary_name(&self) -> String { self.affiliation_id.0.to_string() }
}

#[async_trait::async_trait]
impl Updatable for Affiliations {
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
            SELECT update_signatures FROM affiliations WHERE affiliation_id = $1
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?
           .get::<i64, _>(0);
        Ok(self.update_signatures.0 > may_older)
    }
}

#[async_trait::async_trait]
impl Transactable<Affiliations> for Affiliations {

    /// ### Arguments
    /// * `transaction` - Instances of Postgres database connections.
    async fn insert(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, sqlx::Error> {
        // language=SQL
        let insert: Affiliations = sqlx::query_as::<_, Self>(r#"
            INSERT INTO affiliations (affiliation_id, name, update_signatures)
             VALUES ($1, $2, $3)
            RETURNING *
        "#).bind(self.affiliation_id.0)
           .bind(&self.name)
           .bind(self.update_signatures.0)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(insert)
    }

    /// Update the [Affiliations] data stored in the database.
    ///
    /// ### Return Values
    /// `Result<(Self, Self), Error>` - Result will be returned as the return value.
    /// * `(Affiliations, Affiliations)` - The elements of the tuple are (old-data, new-data).
    /// * `Error` - Error returns the error of sqlx.
    /// ### Arguments
    /// * `transaction` - Instances of Postgres database connections.
    async fn update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self, Self), sqlx::Error> {
        // fixme: It is not a fully parallel process, so I should use tokio::join! (まぁ面倒くさいだけなんだけど。)
        // language=SQL
        let old: Affiliations = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?;
        // language=SQL
        let update: Affiliations = sqlx::query_as::<_, Self>(r#"
            UPDATE affiliations
            SET name = $1, update_signatures = $2
            WHERE affiliation_id = $3
            RETURNING *
        "#).bind(&self.name)
           .bind(self.update_signatures.0)
           .bind(self.affiliation_id.0)
           .fetch_one(&mut *transaction)
           .await?;

        Ok((old, update))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // fixme: It is not a fully parallel process, so I should use tokio::join! (まぁ面(以下略 )
        // language=SQL
        let primary = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM affiliations WHERE name LIKE '$1')
        "#).bind(&self.name)
           .fetch_one(&mut *transaction)
           .await?
           .get::<bool, _>(0);
        // language=SQL
        let secondary = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM affiliations WHERE affiliation_id = $1)
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?
           .get::<bool, _>(0);

        Ok(primary || secondary)
    }

    async fn delete(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<i64, sqlx::Error> {
        // language=SQL
        let del = sqlx::query_as::<_, AffiliationId>(r#"
            DELETE FROM affiliations WHERE affiliation_id = $1 RETURNING affiliation_id
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del.0)
    }
}