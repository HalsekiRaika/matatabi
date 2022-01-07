use dotenv::dotenv;
use logger::Logger;

mod database;
mod models;
mod server;
mod routes;
mod logger;

//#[tokio::main]
#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let logger = Logger::new(Some("Matatabi"));
    logger.info("Cats are crazy about Matatabi. ฅ^•ω•^ฅ");

    database::postgres_database::migration()
        .await
        .expect("An Error occurred by database migration.");
    let pool = database::postgres_database::connect()
        .await
        .expect("An Error occurred by database connection pool.");

    server::server_run(pool).await;

    Ok(())
}