use super::id_object::AffiliationId;
use sqlx::{Error, FromRow, Row, Transaction};
use sqlx::postgres::Postgres;
use crate::database::models::{Printable, Updatable};
use crate::database::transactable::Transactable;
use crate::Logger;
use super::update_signature::UpdateSignature;

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

#[allow(dead_code, unused_variables)]
impl Affiliations {
    pub fn new(id: i64, name: impl Into<String>, update_signatures: i64) -> Affiliations {
        Self { affiliation_id: AffiliationId(id), name: name.into(), update_signatures: UpdateSignature(update_signatures) }
    }

    pub fn get_affiliation_id(&self) -> i64 {
        self.affiliation_id.0.clone()
    }

    pub async fn update_name(
        id: AffiliationId,
        name: impl Into<String>,
        transaction: &mut sqlx::Transaction<'_, Postgres>
    ) -> Result<(), sqlx::Error> {
        let update_sign = UpdateSignature::default();
        sqlx::query!(
            "
            UPDATE affiliations
            SET name = $1, update_signatures = $2
            WHERE affiliation_id = $3
            ",
            &name.into(),
            update_sign as UpdateSignature,
            id as AffiliationId
        )
        .execute(&mut *transaction)
        .await?;
        Ok(())
    }

    pub async fn fetch_id<'a, 'b, E>(
        name: impl Into<String>,
        executor: E
    ) -> Result<Option<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> {
        let name_a = name.into().clone();
        let obj = sqlx::query!(
            "
            SELECT affiliation_id, update_signatures FROM affiliations WHERE name = $1
            ",
            &name_a
        )
        .fetch_optional(executor)
        .await?;

        if let Some(searched) = obj {
            Ok(Some(Affiliations::new(searched.affiliation_id, name_a, searched.update_signatures)))
        } else {
            Ok(None)
        }
    }

    pub async fn fetch_name<'a, 'b, E>(
        id: AffiliationId,
        executor: E
    ) -> Result<Option<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> {
        let obj = sqlx::query!(
            "
            SELECT name, update_signatures FROM affiliations WHERE affiliation_id = $1
            ",
            id as AffiliationId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(searched) = obj {
            Ok(Some(Affiliations::new(id.0, searched.name, searched.update_signatures)))
        } else {
            Ok(None)
        }
    }

    pub async fn fetch_all_id<'a, 'b, E>(
        executor: E
    ) -> Result<Vec<AffiliationId>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> {
        let objs = sqlx::query!(
            "
            SELECT affiliation_id FROM affiliations
            "
        )
        .fetch_all(executor)
        .await?;

        let mut items: Vec<AffiliationId> = Vec::new();
        for item in objs {
            items.push(AffiliationId(item.affiliation_id));
        }

        Ok(items)
    }

    pub async fn exists<'a, 'b, E>(
        &self,
        executor: E
    ) -> Result<bool, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> + Clone {
        let (a, b) = tokio::join!(
            Affiliations::fetch_id(self.name.clone(), executor.clone()),
            Affiliations::fetch_name(self.affiliation_id.clone(), executor.clone())
        );

        let none_aff: Affiliations = Affiliations::new(0, "none_name", UpdateSignature::default().as_i64());

        let (i, j) = (a?.unwrap_or(none_aff.clone()), b?.unwrap_or(none_aff.clone()));
        if (i != none_aff.clone() || j != none_aff.clone()) || (i == j) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Printable for Affiliations {
    fn get_primary_name(&self) -> String {
        self.name.clone()
    }
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

    async fn can_update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        let may_older: i64 = sqlx::query(
            r#"SELECT update_signatures FROM affiliations WHERE affiliation_id = $1"#
        )
        .bind(self.affiliation_id)
        .fetch_one(&mut *transaction)
        .await?
        .get::<i64, _>(0);
        Ok(self.update_signatures.0 >= may_older)
    }
}

#[async_trait::async_trait]
impl Transactable for Affiliations {
    async fn insert(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<(), Error> {
        let logger = Logger::new(Some("Transaction"));
        let insert: Affiliations = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO affiliations (
                affiliation_id, name, update_signatures
            )
            VALUES (
                $1, $2, $3
            )
            RETURNING *
            "#
        )
        .bind(self.affiliation_id.0)
        .bind(&self.name)
        .bind(self.update_signatures.0)
        .fetch_one(&mut *transaction)
        .await?;

        logger.info(&format!("| INSERT   | {} + {}", insert.affiliation_id.0, self.update_signatures.0));
        Ok(())
    }

    async fn update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<(), Error> {
        let logger = Logger::new(Some("Transaction"));
        let old: String = sqlx::query(r#"SELECT name FROM affiliations WHERE affiliation_id = $1"#)
            .bind(self.affiliation_id)
            .fetch_one(&mut *transaction)
            .await?
            .get::<String, _>(0);
        let update: Affiliations = sqlx::query_as::<_, Self>(
            r#"
            UPDATE affiliations
            SET name = $1, update_signatures = $2
            WHERE affiliation_id = $3
            RETURNING *
            "#
        )
        .bind(&self.name)
        .bind(self.update_signatures.0)
        .bind(self.affiliation_id.0)
        .fetch_one(&mut *transaction)
        .await?;

        logger.info(&format!("| UPDATE   | {} : {} > {}", update.affiliation_id.0, old, update.name));
        Ok(())
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, Error> {
        let logger = Logger::new(Some("Transaction"));

        let primary = sqlx::query(
            r#"SELECT EXISTS(SELECT 1 FROM affiliations WHERE name LIKE '$1')"#
        )
        .bind(&self.name)
        .fetch_one(&mut *transaction)
        .await?
        .get::<bool, _>(0);
        let secondary = sqlx::query(
            r#"SELECT EXISTS(SELECT 1 FROM affiliations WHERE affiliation_id = $1)"#
        )
        .bind(self.affiliation_id)
        .fetch_one(&mut *transaction)
        .await?
        .get::<bool, _>(0);

        if primary { logger.warn(&format!("| DUP ERR  | {} : This affiliation name exists. Cannot register the same name.", &self.name)); }
        if secondary { logger.warn(&format!("| DUP ERR  | {} : This affiliation id exists. Cannot register the same id", self.affiliation_id.0)); }

        Ok(primary || secondary)
    }
}