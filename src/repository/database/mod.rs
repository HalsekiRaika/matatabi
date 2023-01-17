mod affiliation;

pub use self::{
    affiliation::PgAffiliationRepository
};


use super::RepositoryError;

type Transaction<'r> = sqlx::Transaction<'r, sqlx::Postgres>;

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        RepositoryError::External(anyhow::Error::new(err))
    }
}