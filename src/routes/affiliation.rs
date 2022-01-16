use actix_web::{HttpRequest, HttpResponse, web};
use futures::StreamExt;
use serde_json::json;
use sqlx::PgPool;
use crate::database::models::affiliation_object::Affiliations;
use crate::database::models::id_object::AffiliationId;
use crate::models::affiliation::Affiliation;
use crate::routes::ApiError;
use crate::models::error::ApiError as ErrorDescription;

#[actix_web::get("affiliations")]
pub async fn get_affiliations(_req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let all = Affiliations::fetch_all(&**pool).await?;
    let aff: Vec<_> = futures::stream::iter(all).filter_map(|db_model| async {
        Some(Affiliation::from(db_model))
    }).collect()
      .await;
    Ok(HttpResponse::Ok().json(aff))
}

#[actix_web::get("{id}")]
pub async fn get_affiliation_from_id(
    _req: HttpRequest,
    id: web::Path<i64>,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ApiError> {
    if let Some(from_id) = Affiliations::fetch_name_from_id(AffiliationId(id.into_inner()), &**pool).await? {
        return Ok(HttpResponse::Ok().json(json!([from_id])))
    }
    Ok(HttpResponse::NotFound().json(ErrorDescription {
        error: "not_found",
        description: "No content was found to response this request."
    }))
}