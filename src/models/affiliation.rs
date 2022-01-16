use serde::{Serialize, Deserialize};
use crate::database::models::affiliation_object::Affiliations;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffiliationId(pub i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affiliation {
    pub affiliation_id: AffiliationId,
    pub name: String
}

impl From<crate::database::models::affiliation_object::Affiliations> for Affiliation {
    /// Use to transform affiliation_object(database model) to Affiliation(WebAPI model)
    fn from(database_obj: Affiliations) -> Self {
        Self {
            affiliation_id: AffiliationId::from(database_obj.get_affiliation_id()),
            name: database_obj.get_name().to_string()
        }
    }
}