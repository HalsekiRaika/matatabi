use crate::entities::{Affiliation, AffiliationId};
use crate::repository::{
    AffiliationRepository,
    adaptor::{
        AffiliationCreate,
        AffiliationUpdate,
        AffiliationDelete
    },
    dto::{
        AffiliationDto,
        CreateAffiliation,
        UpdateAffiliation,
    },

    RepositoryError
};

pub struct CreateInteractor<R> {
    affiliation_repo: R
}

impl<R> CreateInteractor<R> {
    pub fn new(repo: R) -> Self {
        Self { affiliation_repo: repo }
    }
}

#[async_trait::async_trait]
impl<R: AffiliationRepository> AffiliationCreate for CreateInteractor<R> {
    async fn create(&self, create: CreateAffiliation) -> Result<AffiliationDto, RepositoryError> {
        let CreateAffiliation { id, name } = create;
        let aff = Affiliation::new(id, name);

        self.affiliation_repo.create(&aff).await?;
        
        Ok(aff.into())
    }
}

pub struct UpdateInteractor<R> {
    affiliation_repo: R
}

impl<R> UpdateInteractor<R> {
    pub fn new(repo: R) -> Self {
        Self { affiliation_repo: repo }
    }
}

#[async_trait::async_trait]
impl<R: AffiliationRepository> AffiliationUpdate for UpdateInteractor<R> {
    async fn update(&self, update: UpdateAffiliation) -> Result<AffiliationDto, RepositoryError> {
        let UpdateAffiliation { id, name } = update;
        let id: AffiliationId = id.into();

        let mut target = self.affiliation_repo.fetch_from_id(&id).await?
            .ok_or(RepositoryError::NotFound { method: "from_id", entity: "affiliation", id: format!("{:?}", id) })?
            .into_destruct();

        target.name = name;

        let freezed = target.freeze();

        self.affiliation_repo.update(&freezed).await?;

        Ok(freezed.into())
    }
}

pub struct DeleteInteractor<R> {
    affiliation_repo: R
}

impl<R> DeleteInteractor<R> {
    pub fn new(repo: R) -> Self {
        Self { affiliation_repo: repo }
    }
}

#[async_trait::async_trait]
impl<R: AffiliationRepository> AffiliationDelete for DeleteInteractor<R> {
    async fn delete(&self, delete: i64) -> Result<(), RepositoryError> {
        let id = AffiliationId::new(delete);
        self.affiliation_repo.delete(&id).await?;
        Ok(())
    }
}