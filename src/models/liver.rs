use serde::{Serialize, Deserialize};
use crate::database::models::id_object::LiverId;
use crate::database::models::livers_object::LiverObject;

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

impl From<LiverId> for NumId<Liver> {
    fn from(db_id: LiverId) -> Self {
        NumId::new(db_id)
    }
}

impl From<LiverObject> for Liver {
    fn from(obj: LiverObject) -> Self {
        Self {
            liver_id: NumId::from(obj.liver_id()),
            affiliation: obj.affiliation_id().map(NumId::from),
            name: obj.name().to_owned(),
            localized_name: obj.localized_name().to_owned(),
            logo_url: format!("[unimplemented feature] https://reiva.dev/api/resources/logos/{}", i64::from(obj.liver_id()))
        }
    }
}