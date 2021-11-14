use super::id_object::AffiliationId;
use sqlx::postgres::Postgres;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affiliations {
    pub affiliation_id: AffiliationId,
    pub name: String
}

#[allow(dead_code, unused_variables)]
impl Affiliations {
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
            self.affiliation_id.clone() as AffiliationId,
            &self.name
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn update_name(
        id: AffiliationId,
        name: String, 
        transaction: &mut sqlx::Transaction<'_, Postgres>
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            UPDATE affiliations 
            SET name = $1
            WHERE affiliation_id = $2
            ",
            &name,
            id as AffiliationId
        )
        .execute(&mut *transaction)
        .await?;
        Ok(())
    }

    pub async fn get_id<'a, 'b, E>(
        name: String,
        executor: E
    ) -> Result<Option<Self>, sqlx::Error> 
      where E: sqlx::Executor<'a, Database = Postgres> {
        let obj = sqlx::query!(
            "
            SELECT affiliation_id FROM affiliations WHERE name = $1
            ",
            name.clone()
        )
        .fetch_optional(executor)
        .await?;

        if let Some(searched) = obj {
            Ok(Some(Affiliations {
                affiliation_id: AffiliationId(searched.affiliation_id),
                name
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_name<'a, 'b, E>(
        id: AffiliationId,
        executor: E
    ) -> Result<Option<Self>, sqlx::Error>
      where E: sqlx::Executor<'a, Database = Postgres> {
        let obj = sqlx::query!(
            "
            SELECT name FROM affiliations WHERE affiliation_id = $1
            ",
            id.clone() as AffiliationId
        )
        .fetch_optional(executor)
        .await?;

        if let Some(searched) = obj {
            Ok(Some(Affiliations {
                affiliation_id: id,
                name: searched.name
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_id<'a, 'b, E>(
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
        for obj in objs {
            items.push(AffiliationId(obj.affiliation_id));
        }
        
        Ok(items)
    }

    pub async fn exists<'a, 'b, E>(
        &self,
        executor: E
    ) -> Result<bool, sqlx::Error> 
      where E: sqlx::Executor<'a, Database = Postgres> + Clone {
        let (a, b) = tokio::join!(
            Affiliations::get_id(self.name.clone(), executor.clone()),
            Affiliations::get_name(self.affiliation_id.clone(), executor.clone())
        );
        let none_aff: Affiliations = Affiliations { 
            affiliation_id: AffiliationId("none".to_string()),
            name: "none_name".to_string() 
        };
        let (i, j) = (a?.unwrap_or(none_aff.clone()), b?.unwrap_or(none_aff.clone()));
        if (i != none_aff.clone() || j != none_aff.clone()) || (i == j) { 
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
