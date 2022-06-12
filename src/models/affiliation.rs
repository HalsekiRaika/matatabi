use serde::{Serialize, Deserialize};
use crate::database::models::affiliation_object::Affiliations as FromDatabaseAffiliation;
use crate::database::models::id_object::AffiliationId as FromDatabaseAffiliationId;

use super::NumId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affiliation {
    pub affiliation_id: NumId<Affiliation>,
    pub name: String
}

impl From<FromDatabaseAffiliationId> for NumId<Affiliation> {
    fn from(database: FromDatabaseAffiliationId) -> Self {
        NumId::new(database.0)
    }
}

impl From<FromDatabaseAffiliation> for Affiliation {
    fn from(database_obj: FromDatabaseAffiliation) -> Self {
        Self {
            affiliation_id: NumId::from(database_obj.get_affiliation_id()),
            name: database_obj.get_name().to_string()
        }
    }
}