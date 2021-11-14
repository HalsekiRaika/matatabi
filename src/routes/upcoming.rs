use actix_web::{get};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

use crate::routes::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelId {
    id: String,
}

// #[get("upcoming-lives")]
// pub async fn upcomings(
//     request: 
// ) -> Result<HttpResponse, ApiError> {
    
// }