#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use sqlx::{Error, Row, Transaction};
use sqlx::postgres::Postgres;

use super::{Accessor, hash, Fetch};
use super::id_object::AffiliationId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::FromRow)]
pub struct AffiliationObject {
    affiliation_id: AffiliationId,
    name: String,
}

impl Display for AffiliationObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "affiliation >> {}, name: {}", self.affiliation_id, self.name)
    }
}

impl AffiliationObject {
    pub fn new(id: impl Into<i64>, name: impl Into<String>) -> AffiliationObject {
        Self { affiliation_id: AffiliationId::new(id), name: name.into() }
    }

    pub fn affiliation_id(&self) -> AffiliationId { self.affiliation_id }
    pub fn name(&self) -> &str { &self.name }
}

impl AffiliationObject {
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
        id: impl Into<i64>,
        transaction: E
    ) -> Result<Option<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let searched = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#).bind(id.into())
           .fetch_optional(transaction)
           .await?;

        Ok(searched)
    }
}

#[async_trait::async_trait]
impl Fetch for AffiliationObject {
    async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

#[async_trait::async_trait]
impl Accessor for AffiliationObject {
    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, Error> {
        // language=SQL
        let ins = sqlx::query_as::<_, Self>(r#"
            INSERT INTO affiliations (affiliation_id, name)
             VALUES ($1, $2)
            RETURNING *
        "#).bind(self.affiliation_id)
           .bind(&self.name)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(ins)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, Error> {
        // language=SQL
        let del = sqlx::query_as::<_, Self>(r#"
            DELETE FROM affiliations WHERE affiliation_id = $1 RETURNING *
        "#).bind(self.affiliation_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self, Self), Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#).bind(self.affiliation_id)
            .fetch_one(&mut *transaction)
            .await?;
        // language=SQL
        let update = sqlx::query_as::<_, Self>(r#"
            UPDATE affiliations SET name = $1 WHERE affiliation_id = $2
            RETURNING *
        "#).bind(&self.name)
           .bind(self.affiliation_id)
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

    async fn compare(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, Error> {
        let com = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#).bind(&self.affiliation_id)
           .fetch_optional(&mut *transaction)
           .await?;
        
        let com = if let Some(db) = com {
            let db = hash(&db);
            let my = hash(&self);
            db == my
        } else { false };
        Ok(com)
    }
}