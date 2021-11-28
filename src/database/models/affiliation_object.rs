use super::id_object::AffiliationId;
use sqlx::postgres::Postgres;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affiliations {
    pub affiliation_id: AffiliationId,
    pub name: String
}

#[allow(dead_code, unused_variables)]
impl Affiliations {
    pub fn new(id: i64, name: impl Into<String>) -> Affiliations {
        Self { affiliation_id: AffiliationId(id), name: name.into() }
    }

    pub async fn insert(
        &self, transaction: &mut sqlx::Transaction<'_, Postgres>
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO affiliations (
                affiliation_id, name
            )
            VALUES (
                $1, $2
            )
            ",
            self.affiliation_id as AffiliationId,
            &self.name
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn update_name(
        id: AffiliationId,
        name: impl Into<String>,
        transaction: &mut sqlx::Transaction<'_, Postgres>
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            UPDATE affiliations
            SET name = $1
            WHERE affiliation_id = $2
            ",
            &name.into(),
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
            SELECT affiliation_id FROM affiliations WHERE name = $1
            ",
            &name_a
        )
        .fetch_optional(executor)
        .await?;

        if let Some(searched) = obj {
            Ok(Some(Affiliations::new(0 /*searched.affiliation_id*/, name_a)))
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
            SELECT name FROM affiliations WHERE affiliation_id = $1
            ",
            id as AffiliationId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(searched) = obj {
            Ok(Some(Affiliations::new(id.0, searched.name)))
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

        let none_aff: Affiliations = Affiliations::new(0, "none_name");

        let (i, j) = (a?.unwrap_or(none_aff.clone()), b?.unwrap_or(none_aff.clone()));
        if (i != none_aff.clone() || j != none_aff.clone()) || (i == j) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
