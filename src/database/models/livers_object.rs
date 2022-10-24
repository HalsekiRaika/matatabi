#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use sqlx::{Error, Postgres, Row, Transaction};

use super::{Accessor, hash, Fetch};
use super::id_object::{AffiliationId, LiverId};

#[derive(Debug, Clone, PartialEq, Hash, Eq, sqlx::FromRow)]
pub struct LiverObject {
    liver_id: LiverId,
    affiliation_id: Option<AffiliationId>,
    name: String,
    localized_name: String,
}

impl Display for LiverObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "liver >> {}, affiliation(id): {:?}, name: {}", self.liver_id, self.affiliation_id, self.name)
    }
}

impl LiverObject {
    pub fn new(
        id: impl Into<i64>, affiliation_id: impl Into<Option<i64>>,
        name: impl Into<String>, localized_name: impl Into<String>
    ) -> Self {
        Self {
            liver_id: LiverId::new(id.into()), 
            affiliation_id: affiliation_id.into().map(AffiliationId::new),
            name: name.into(), localized_name: localized_name.into()
        }
    }

    pub fn name(&self) -> &str {
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

impl LiverObject {
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

#[async_trait::async_trait]
impl Fetch for LiverObject {
    async fn fetch_all<'a, E>(transaction: E) -> Result<Vec<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Copy {
        // language=SQL
        let all = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM livers
        "#).fetch_all(transaction)
           .await?;
        Ok(all)
    }
}

#[async_trait::async_trait]
impl Accessor for LiverObject {
    async fn insert(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, Error> {
        // language=SQL
        let ins = sqlx::query_as::<_, Self>(r#"
            INSERT INTO livers (liver_id, affiliation_id, name, localized_name)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#).bind(self.liver_id)
           .bind(self.affiliation_id)
           .bind(&self.name)
           .bind(&self.localized_name)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(ins)
    }

    async fn delete(self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, Error> {
        // language=SQL
        let del = sqlx::query_as::<_, Self>(r#"
            DELETE FROM livers WHERE liver_id = $1 RETURNING *
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del)
    }

    async fn update(self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self, Self), Error> {
        // language=SQL
        let old = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM livers WHERE liver_id = $1
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        // language=SQL
        let update = sqlx::query_as::<_, Self>(r#"
            UPDATE livers SET name = $1, affiliation_id = $2 WHERE liver_id = $3
            RETURNING *
        "#).bind(&self.name)
           .bind(self.affiliation_id)
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

    async fn compare(&self, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<bool, sqlx::Error> {
        let com = sqlx::query_as::<_, Self>(r#"
            SELECT * FROM livers WHERE liver_id = $1
        "#).bind(&self.liver_id)
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