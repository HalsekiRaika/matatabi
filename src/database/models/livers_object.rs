#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use sqlx::{Error, Postgres, Row, Transaction};

use super::Transact;
use super::id_object::{AffiliationId, LiverId};
use super::update_signature::{UpdateSignature, LatestEq, Signed, Version};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Livers {
    liver_id: LiverId,
    affiliation_id: Option<AffiliationId>,
    name: String,
    update_signatures: UpdateSignature
}

impl Display for Livers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "liver >> {}, affiliation(id): {:?}, name: {}", self.liver_id, self.affiliation_id, self.name)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct RawLivers {
    liver_id: Option<i64>,
    affiliation_id: Option<i64>,
    name: Option<String>,
    update_signatures: Option<i64>
}

impl From<RawLivers> for Livers {
    fn from(raw: RawLivers) -> Self {
        let id = if let Some(id) = raw.liver_id { id } else { 0 };
        let aff = raw.affiliation_id;
        let name = if let Some(name) = raw.name { name } else { "none".to_string() };
        let sign = if let Some(sign) = raw.update_signatures { sign } else { 0 };
        Livers::new(id, aff, name, sign)
    }
}

impl Livers {
    pub fn new(
        liver_id: i64, affiliation_id: Option<i64>,
        name: impl Into<String>, update_signature: i64
    ) -> Self {
        let aff = affiliation_id.map(AffiliationId);
        Self {
            liver_id: LiverId(liver_id), affiliation_id: aff,
            name: name.into(), update_signatures: UpdateSignature(update_signature)
        }
    }

    pub fn as_ref_name(&self) -> &str {
        &self.name
    }

    pub fn liver_id(&self) -> LiverId {
        self.liver_id
    }

    pub fn affiliation_id(&self) -> Option<AffiliationId> {
        self.affiliation_id
    }
}

impl Livers {
    pub async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
        where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM livers
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

impl Version for Livers {
    fn version(&self) -> UpdateSignature {
        self.update_signatures
    }
}

#[async_trait::async_trait]
impl Signed for Livers {
    async fn sign(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<UpdateSignature, Error> {
        // language=SQL
        let current = sqlx::query(r#"
            SELECT update_signatures FROM livers WHERE liver_id = $1
        "#).bind(&self.liver_id)
           .fetch_one(&mut *transaction)
           .await?
           .try_get::<UpdateSignature, _>(0)?;
        Ok(current)
    }
}

impl LatestEq for Livers {
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
impl Transact for Livers {
    type TransactItem = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::TransactItem, Error> {
        // language=SQL
        let ins: Livers = sqlx::query_as::<_, Self>(r#"
            INSERT INTO livers (liver_id, affiliation_id, name, update_signatures)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#).bind(self.liver_id)
           .bind(self.affiliation_id)
           .bind(&self.name)
           .bind(self.update_signatures)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(ins)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::TransactItem, Error> {
        // language=SQL
        let del = sqlx::query_as::<_, Self>(r#"
            DELETE FROM livers WHERE liver_id = $1 RETURNING *
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self::TransactItem, Self::TransactItem), Error> {
        // language=SQL
        let old: Livers = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM livers WHERE liver_id = $1
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        // language=SQL
        let update: Livers = sqlx::query_as::<_, Self>(r#"
            UPDATE livers
            SET name = $1, affiliation_id = $2, update_signatures = $3
            WHERE liver_id = $4
            RETURNING *
        "#).bind(&self.name)
           .bind(self.affiliation_id)
           .bind(self.update_signatures)
           .bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok((old, update))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, Error> {
        // language=SQL
        let is_name_exist = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM livers WHERE name LIKE $1)
        "#).bind(&self.name)
           .fetch_one(&mut *transaction)
           .await?
           .try_get::<bool, _>(0)?;
        // language=SQL
        let is_id_exist = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM livers WHERE liver_id = $1)
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?
           .try_get::<bool, _>(0)?;
        Ok(is_name_exist || is_id_exist)
    }
}