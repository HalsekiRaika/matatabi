use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: &'a str
}