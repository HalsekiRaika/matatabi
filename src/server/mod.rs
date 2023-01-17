mod salmon;

mod axum;
mod layer;

#[allow(unused_must_use)]
pub async fn server_run(pool: sqlx::PgPool) {
    salmon::run_salmon(pool.clone()).await;
    axum::run_webapi_server(pool.clone()).await;
}
