pub mod actix;
pub mod middleware;
pub mod salmon;
pub mod meilisearch;

#[allow(unused_must_use)]
pub async fn server_run(pool: sqlx::PgPool) {
    salmon::run_salmon(pool.clone()).await;
    actix::run_actix(pool.clone()).await;
}