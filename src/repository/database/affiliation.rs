use futures::{StreamExt, TryStreamExt, TryFutureExt};
use sqlx::{Pool, Postgres};

use crate::entities::{Affiliation, AffiliationId};

use crate::repository::{
    AffiliationRepository,
    RepositoryError,
};

use super::Transaction;

#[derive(Debug, Clone)]
pub struct PgAffiliationRepository {
    pool: Pool<Postgres>
}

impl PgAffiliationRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AffiliationRepository for PgAffiliationRepository {
    async fn create(&self, create: &Affiliation) -> Result<(), RepositoryError> {
        let mut transaction = self.pool.begin().await?;
        if let Err(internal) = Internal::insert(create, &mut transaction).await {
            transaction.rollback().await?;
            return Err(internal.into());
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn update(&self, update: &Affiliation) -> Result<(), RepositoryError> {
        let mut transaction = self.pool.begin().await?;
        if let Err(internal) = Internal::update(update, &mut transaction).await {
            transaction.rollback().await?;
            return Err(internal.into());
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn delete(&self, delete: &AffiliationId) -> Result<(), RepositoryError> {
        let mut transaction = self.pool.begin().await?;
        if let Err(internal) = Internal::delete(delete, &mut transaction).await {
            transaction.rollback().await?;
            return Err(internal.into());
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn fetch_all(&self) -> Result<Vec<Affiliation>, RepositoryError> {
        let mut transaction = self.pool.begin().await?;
        let all  = Internal::select_all(&mut transaction).await?;
        Ok(all)
    }

    async fn fetch_from_id(&self, id: &AffiliationId) -> Result<Option<Affiliation>, RepositoryError> {
        let mut transaction = self.pool.begin().await?;
        let aff = Internal::select_from_id(id, &mut transaction).await?;
        Ok(aff)
    }

    async fn fetch_from_name(&self, name: &str) -> Result<Option<Affiliation>, RepositoryError> {
        let mut transaction = self.pool.begin().await?;
        let aff = Internal::select_from_name(name, &mut transaction).await?;
        Ok(aff)
    }
}

#[derive(sqlx::FromRow)]
struct AffiliationRow {
    pub affiliation_id: i64,
    pub name: String,
}

struct Internal;

impl Internal {
    async fn insert(create: &Affiliation, transaction: &mut Transaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            INSERT INTO affiliations (
                affiliation_id,
                name
            )
            VALUES ($1, $2)
        "#)
        .bind(create.id().as_ref())
        .bind(create.name())
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    async fn update(update: &Affiliation, transaction: &mut Transaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            UPDATE SET
                name = $1
            WHERE id = $2
        "#)
        .bind(update.name())
        .bind(update.id().as_ref())
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    async fn delete(delete: &AffiliationId, transaction: &mut Transaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            DELETE FROM affiliations WHERE affiliation_id = $1
        "#)
        .bind(delete.as_ref())
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    async fn select_all(transaction: &mut Transaction<'_>) -> Result<Vec<Affiliation>, sqlx::Error> {
        let all: Vec<Affiliation> = sqlx::query_as(r#"
            SELECT * FROM affiliations
        "#)
        .fetch(&mut *transaction)
        .map(|row: Result<AffiliationRow, sqlx::Error>| -> Result<Affiliation, sqlx::Error> {
            let row = row?;
            Ok(Affiliation::new(row.affiliation_id, row.name))
        })
        .try_collect()
        .await?;
        
        Ok(all)
    }

    async fn select_from_id(id: &AffiliationId, transaction: &mut Transaction<'_>) -> Result<Option<Affiliation>, sqlx::Error> {
        let aff: Option<Affiliation> = sqlx::query_as(r#"
            SELECT * FROM affiliations WHERE affiliation_id = $1
        "#)
        .bind(id.as_ref())
        .fetch_optional(&mut *transaction)
        .and_then(|row: Option<AffiliationRow>| async { 
            Ok(row.map(|row| Affiliation::new(row.affiliation_id, row.name))) 
        })
        .await?;
        
        Ok(aff)
    }

    async fn select_from_name(name: &str, transaction: &mut Transaction<'_>) -> Result<Option<Affiliation>, sqlx::Error> {
        let aff: Option<Affiliation> = sqlx::query_as(r#"
            SELECT * FROM affiliations WHERE name LIKE $1
        "#)
        .bind(name)
        .fetch_optional(&mut *transaction)
        .and_then(|row: Option<AffiliationRow>| async { 
            Ok(row.map(|row| Affiliation::new(row.affiliation_id, row.name))) 
        })
        .await?;

        Ok(aff)
    }
}