pub mod actix;
pub mod auth;
pub mod yarn;

#[actix_rt::main]
pub async fn server_run(pool: sqlx::PgPool) {
    let (_a, _b) = tokio::join!(
        actix::run_actix(pool.clone()),
        yarn::run_yarn(pool.clone())
    );
}