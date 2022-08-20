#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use sqlx::{Error, FromRow, Row, Transaction};
use sqlx::postgres::Postgres;

use super::Accessor;
use super::id_object::AffiliationId;
use super::update_signature::{UpdateSignature, Version, Signed, LatestEq};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Affiliations {
    affiliation_id: AffiliationId,
    name: String,
    update_signatures: UpdateSignature
}

impl Display for Affiliations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "affiliation >> {}, name: {}", self.affiliation_id, self.name)
    }
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
        id: i64,
        transaction: E
    ) -> Result<Option<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let searched = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#).bind(id)
           .fetch_optional(transaction)
           .await?;

        Ok(searched)
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

impl Version for Affiliations {
    fn version(&self) -> UpdateSignature {
        self.update_signatures
    }
}

#[async_trait::async_trait]
impl Signed for Affiliations {
    async fn sign(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<UpdateSignature, Error> {
        // language=SQL
        let current = sqlx::query(r#"
            SELECT update_signatures FROM affiliations WHERE affiliation_id = $1
        "#).bind(self.affiliation_id)
            .fetch_one(&mut *transaction)
            .await?
            .try_get::<UpdateSignature, _>(0)?;
        Ok(current)
    }
}

impl LatestEq for Affiliations {
    type ComparisonItem = Self;

    fn apply(self, sign: UpdateSignature) -> Self::ComparisonItem {
        let mut a = self;
        a.update_signatures = sign;
        a
    }

    fn version_compare(&self, compare: UpdateSignature) -> bool {
        self.update_signatures.0 > compare.0
    }

    fn irregular_sign(&self) -> bool {
        self.update_signatures.0 <= 1
    }
}

#[async_trait::async_trait]
impl Accessor for Affiliations {
    type Item = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let ins = sqlx::query_as::<_, Self>(r#"
            INSERT INTO affiliations (affiliation_id, name, update_signatures)
             VALUES ($1, $2, $3)
            RETURNING *
        "#).bind(self.affiliation_id.0)
           .bind(&self.name)
           .bind(self.update_signatures.0)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(ins)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let del = sqlx::query_as::<_, Self>(r#"
            DELETE FROM affiliations WHERE affiliation_id = $1 RETURNING *
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self::Item, Self::Item), Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#).bind(self.affiliation_id)
            .fetch_one(&mut *transaction)
            .await?;
        // language=SQL
        let update = sqlx::query_as::<_, Self>(r#"
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

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, Error> {
        // language=SQL
        let primary = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM affiliations WHERE name LIKE '$1')
        "#).bind(&self.name)
           .fetch_one(&mut *transaction)
           .await?
           .try_get::<bool, _>(0)?;
        // language=SQL
        let secondary = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM affiliations WHERE affiliation_id = $1)
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?
           .try_get::<bool, _>(0)?;

        Ok(primary || secondary)
    }
}
