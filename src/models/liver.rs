use serde::{Serialize, Deserialize};
use crate::database::models::id_object::LiverId as FromDatabaseLiverId;
use crate::database::models::livers_object::Livers as FromDatabaseLiver;

use super::NumId;
use super::affiliation::Affiliation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liver {
    pub liver_id: NumId<Liver>,
    pub affiliation: Option<NumId<Affiliation>>,
    pub name: String,
    pub localized_name: String,
    pub logo_url: String,
}

impl From<FromDatabaseLiverId> for NumId<Liver> {
    fn from(database: FromDatabaseLiverId) -> Self {
        NumId::new(database.0)
    }
}

impl From<FromDatabaseLiver> for Liver {
    fn from(database_obj: FromDatabaseLiver) -> Self {
        Self {
            liver_id: NumId::from(database_obj.liver_id()),
            affiliation: database_obj.affiliation_id().map(NumId::from),
            name: database_obj.as_ref_name().to_string(),
            localized_name: database_obj.localized_name().to_string(),
            logo_url: format!("https://reiva.dev/api/resources/logos/{}", database_obj.liver_id().0)
        }
    }
}