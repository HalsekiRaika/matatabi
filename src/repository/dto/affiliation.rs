use crate::entities::{Affiliation, DestructAffiliation};

#[derive(Debug)]
pub struct AffiliationDto {
    pub id: i64,
    pub name: String,
}

impl From<Affiliation> for AffiliationDto {
    fn from(entity: Affiliation) -> Self {
        let DestructAffiliation { id, name } = entity.into_destruct();
        Self { id: id.into(), name }
    }
}

pub struct CreateAffiliation {
    pub id: i64,
    pub name: String
}

pub struct UpdateAffiliation {
    pub id: i64,
    pub name: String
}