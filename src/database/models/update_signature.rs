use sqlx::{Type, FromRow, Postgres, Transaction};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Type, FromRow)]
#[sqlx(transparent)]
pub struct UpdateSignature(pub i64);

impl UpdateSignature {
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Default for UpdateSignature {
    fn default() -> Self {
        Self(Utc::now()
            .format("%Y%m%d%H%M")
            .to_string()
            .parse()
            .unwrap())
    }
}

pub trait Version {
    fn version(&self) -> UpdateSignature;
}

pub trait LatestEq: Version {
    type ComparisonItem;

    fn apply(self, sign: UpdateSignature) -> Self::ComparisonItem;
    fn version_compare(&self, compare: UpdateSignature) -> bool;
    fn irregular_sign(&self) -> bool;
}

#[async_trait::async_trait]
pub trait Signed {
    async fn sign(&self, transaction: &mut Transaction<'_, Postgres>) -> Result<UpdateSignature, sqlx::Error>;
}