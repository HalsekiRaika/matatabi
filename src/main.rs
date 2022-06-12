use dotenv::dotenv;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod database;
mod models;
mod server;
mod routing;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "matatabi=debug".into())))
        .with(tracing_subscriber::fmt::layer())
        .init();
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    tracing::info!("Cats are crazy about Matatabi. ฅ^•ω•^ฅ");

    database::postgres_database::migration()
        .await
        .expect("An Error occurred by database migration.");
    let pool = database::postgres_database::connect()
        .await
        .expect("An Error occurred by database connection pool.");

    server::server_run(pool).await;

    Ok(())
}