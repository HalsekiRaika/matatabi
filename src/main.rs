use dotenv::dotenv;
use logger::Logger;

mod database;
mod models;
mod server;
mod routes;
mod logger;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    let logger = Logger::new(Some("Matatabi"));
    logger.info("Cats are crazy about Matatabi. ฅ^•ω•^ฅ");

    database::mysql_database::migration()
        .await
        .expect("An Error occured by database migration.");
    let pool = database::mysql_database::connect()
        .await
        .expect("An Error occured by database connection pool.");

    server::actix::run_actix(pool)
        .expect("Failed to run webapi server thread.");
    Ok(())
}