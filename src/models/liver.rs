use serde::{Serialize, Deserialize};
use crate::database::models::livers_object::Livers;

use super::affiliation::AffiliationId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiverId(pub i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liver {
    pub liver_id: LiverId,
    pub affiliation: Option<AffiliationId>,
    pub logo_url: String,
}

impl From<crate::database::models::livers_object::Livers> for Liver {
    fn from(database_obj: Livers) -> Self {
        let aff = database_obj.get_affiliation_id().map(AffiliationId::from);

        Self {
            liver_id: LiverId::from(database_obj.get_liver_id()),
            affiliation: aff,
            logo_url: format!("https://reiva.dev/api/resources/logos/{}", database_obj.get_liver_id().0)
        }
    }
}