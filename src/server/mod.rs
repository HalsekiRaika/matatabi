pub mod actix;
pub mod middleware;
pub mod salmon;
pub mod meilisearch;
mod axum;

#[allow(unused_must_use)]
pub async fn server_run(pool: sqlx::PgPool) {
    salmon::run_salmon(pool.clone()).await;
//  actix::run_actix(pool.clone()).await;
    axum::run_webapi_server(pool.clone()).await;
}
