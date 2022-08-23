use serde::{Serialize, Deserialize};
use crate::database::models::affiliation_object::AffiliationObject;
use crate::database::models::id_object::AffiliationId;

use super::NumId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affiliation {
    pub affiliation_id: NumId<Affiliation>,
    pub name: String
}

impl From<AffiliationId> for NumId<Affiliation> {
    fn from(db_id: AffiliationId) -> Self {
        NumId::new(db_id)
    }
}

impl From<AffiliationObject> for Affiliation {
    fn from(obj: AffiliationObject) -> Self {
        Self {
            affiliation_id: NumId::from(obj.affiliation_id()),
            name: obj.name().to_owned()
        }
    }
}