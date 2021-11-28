use serde::{Serialize, Deserialize};

use super::affiliation::AffiliationId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiverId(pub i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liver {
    pub liver_id: LiverId,
    pub affiliation: Option<AffiliationId>,
    pub logo_url: String,
}