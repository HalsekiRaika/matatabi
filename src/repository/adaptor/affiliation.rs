use crate::repository::dto::AffiliationDto;
use crate::repository::{RepositoryError, CreateAffiliation, UpdateAffiliation};

#[async_trait::async_trait]
pub trait AffiliationCreate: Send + Sync + 'static {
    async fn create(&self, create: CreateAffiliation) -> Result<AffiliationDto, RepositoryError>;
}

#[async_trait::async_trait]
pub trait AffiliationUpdate: Send + Sync + 'static {
    async fn update(&self, update: UpdateAffiliation) -> Result<AffiliationDto, RepositoryError>;
}

#[async_trait::async_trait]
pub trait AffiliationDelete: Send + Sync + 'static {
    async fn delete(&self, delete: i64) -> Result<(), RepositoryError>;
}