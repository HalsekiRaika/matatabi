use dotenv::dotenv;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[deprecated]
mod database;

mod postgres;
mod repository;

#[deprecated]
mod models;

mod entities;
mod server;
mod routing;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    let appender = tracing_appender::rolling::daily(std::path::Path::new("./logs/"), "debug.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_filter(tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "matatabi=debug".into())))
            .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG))
        .with(tracing_subscriber::fmt::Layer::default()
            .with_writer(non_blocking_appender)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG))
        .init();

    tracing::info!("Cats are crazy about Matatabi. ฅ^•ω•^ฅ");

    postgres::migration()
        .await
        .expect("An Error occurred by database migration.");
    let pool = postgres::connect()
        .await
        .expect("An Error occurred by database connection pool.");

    server::server_run(pool).await;

    Ok(())
}