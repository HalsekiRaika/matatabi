use actix_web::{Responder, HttpResponse};
use crate::models::error::ApiError;

pub async fn nf() -> impl Responder {
    HttpResponse::NotFound().json(ApiError {
        error: "not_found",
        description: "No content was found to response this request."
    })
}