use crate::entities::{Affiliation, AffiliationId};
use crate::repository::RepositoryError;

#[async_trait::async_trait]
pub trait AffiliationRepository: Send + Sync + 'static {
    async fn create(&self, create: &Affiliation) -> Result<(), RepositoryError>;
    async fn update(&self, update: &Affiliation) -> Result<(), RepositoryError>;
    async fn delete(&self, delete: &AffiliationId) -> Result<(), RepositoryError>;
    async fn fetch_all(&self) -> Result<Vec<Affiliation>, RepositoryError>;
    async fn fetch_from_id(&self, id: &AffiliationId) -> Result<Option<Affiliation>, RepositoryError>;
    async fn fetch_from_name(&self, name: &str) -> Result<Option<Affiliation>, RepositoryError>;
}