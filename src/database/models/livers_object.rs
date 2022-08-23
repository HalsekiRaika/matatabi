#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use sqlx::{Error, Postgres, Row, Transaction};

use super::Accessor;
use super::id_object::{AffiliationId, LiverId};
use super::update_signature::{UpdateSignature, LatestEq, Signed, Version};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Livers {
    liver_id: LiverId,
    affiliation_id: Option<AffiliationId>,
    name: String,
    localized_name: String,
    update_signatures: UpdateSignature
}

impl Display for Livers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "liver >> {}, affiliation(id): {:?}, name: {}", self.liver_id, self.affiliation_id, self.name)
    }
}

impl LiverObject {
    pub fn new(
        liver_id: i64, affiliation_id: Option<i64>,
        name: impl Into<String>, localized_name: impl Into<String>, update_signature: i64
    ) -> Self {
        let aff = affiliation_id.map(AffiliationId);
        Self {
            liver_id: LiverId(liver_id), affiliation_id: aff,
            name: name.into(), localized_name: localized_name.into(), update_signatures: UpdateSignature(update_signature)
        }
    }

    pub fn as_ref_name(&self) -> &str {
        &self.name
    }

    pub fn localized_name(&self) -> &str { &self.localized_name }

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

    pub async fn fetch_filtered_affiliation<'a, E>(id: i64, transaction: E) -> Result<Vec<Self>, sqlx::Error>
        where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let filtered = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM livers WHERE affiliation_id = $1
        "#).bind(id)
           .fetch_all(transaction)
           .await?;
        Ok(filtered)
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
impl Accessor for Livers {
    type Item = Self;

    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let ins: Livers = sqlx::query_as::<_, Self>(r#"
            INSERT INTO livers (liver_id, affiliation_id, name, localized_name, update_signatures)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
        "#).bind(self.liver_id)
           .bind(self.affiliation_id)
           .bind(&self.name)
           .bind(&self.localized_name)
           .bind(self.update_signatures)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(ins)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self::Item, Error> {
        // language=SQL
        let del = sqlx::query_as::<_, Self>(r#"
            DELETE FROM livers WHERE liver_id = $1 RETURNING *
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self::Item, Self::Item), Error> {
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