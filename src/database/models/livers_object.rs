use sqlx::{Postgres, Row, Transaction};
use crate::database::models::id_object::{AffiliationId, LiverId};
use crate::database::models::{Printable, Transactable, Updatable};
use crate::database::models::update_signature::UpdateSignature;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Livers {
    liver_id: LiverId,
    affiliation_id: Option<AffiliationId>,
    name: String,
    update_signatures: UpdateSignature
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

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_liver_id(&self) -> LiverId {
        self.liver_id
    }

    pub fn get_affiliation_id(&self) -> Option<AffiliationId> {
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

impl Printable for Livers {
    fn get_primary_name(&self) -> String {
        self.name.clone()
    }

    fn get_secondary_name(&self) -> String {
        self.liver_id.0.to_string()
    }
}

#[async_trait::async_trait]
impl Updatable for Livers {
    fn apply_signature(&self, sign: i64) -> Self {
        let mut a = self.clone();
        a.update_signatures = UpdateSignature(sign);
        a
    }

    fn is_empty_sign(&self) -> bool {
        self.update_signatures.0 <= 1
    }

    fn get_signature(&self) -> i64 {
        self.update_signatures.0
    }

    async fn can_update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let may_older: i64 = sqlx::query(r#"
            SELECT update_signatures FROM livers WHERE liver_id = $1
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?
           .get::<i64, _>(0);
        Ok(self.update_signatures.0 > may_older)
    }
}

#[async_trait::async_trait]
impl Transactable<Livers> for Livers {
    async fn insert(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<Self, sqlx::Error> {
        // language=SQL
        let insert: Livers = sqlx::query_as::<_, Self>(r#"
            INSERT INTO livers (liver_id, affiliation_id, name, update_signatures)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#).bind(self.liver_id)
           .bind(self.affiliation_id)
           .bind(&self.name)
           .bind(self.update_signatures.0)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(insert)
    }

    async fn update(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<(Self, Self), sqlx::Error> {
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
           .bind(self.affiliation_id.unwrap().0)
           .bind(self.update_signatures.0)
           .bind(self.liver_id.0)
           .fetch_one(&mut *transaction)
           .await?;
        Ok((old, update))
    }

    async fn exists(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<bool, sqlx::Error> {
        // language=SQL
        let is_name_exist = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM livers WHERE name LIKE $1)
        "#).bind(&self.name)
           .fetch_one(&mut *transaction)
           .await?
           .get::<bool, _>(0);
        // language=SQL
        let is_id_exist = sqlx::query(r#"
            SELECT EXISTS(SELECT 1 FROM livers WHERE liver_id = $1)
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?
           .get::<bool, _>(0);
        Ok(is_name_exist || is_id_exist)
    }

    async fn delete(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<i64, sqlx::Error> {
        // language=SQL
        let del = sqlx::query_as::<_, LiverId>(r#"
            DELETE FROM livers WHERE liver_id = $1 RETURNING liver_id
        "#).bind(self.liver_id)
           .fetch_one(&mut *transaction)
           .await?;
        Ok(del.0)
    }
}