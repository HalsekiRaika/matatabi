use sqlx::Type;
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct UpdateSignature(pub i64);

impl UpdateSignature {
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Default for UpdateSignature {
    fn default() -> Self {
        let date = Utc::now();
        let a: i64 = date.format("%Y%m%d%H%M")
            .to_string()
            .parse()
            .unwrap();
        Self { 0: a,}
    }
}