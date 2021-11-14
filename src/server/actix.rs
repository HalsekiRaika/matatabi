use actix_web::{App, web, HttpServer};
use sqlx::PgPool;
use crate::routes::nf;
use crate::Logger;

#[actix_rt::main]
pub async fn run_actix(pool: PgPool) -> Result<(), std::io::Error> {
    let logger = Logger::new(Some("Actix"));
    logger.info("Starting Actix Web Server!");
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .default_service(web::get().to(nf::nf))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}