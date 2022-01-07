use actix_cors::Cors;
use actix_web::{App, web, HttpServer};
use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use sqlx::PgPool;
use tonic::transport::Channel;
use crate::routes::index;
use crate::Logger;
use crate::server::middleware::bearer::Credentials;
use crate::server::middleware::cage;
use crate::server::middleware::cage::cage_api_client::CageApiClient;
use crate::server::middleware::cage_middleware::{CageAuth, CageMiddlewareTaskResult};

//#[actix_rt::main]
pub async fn run_actix(pool: PgPool) -> Result<(), std::io::Error> {
    let logger = Logger::new(Some("Actix"));
    logger.info("Starting Actix Web Server!");

    let verification_server_conf = CageApiClient::connect("http://[::1]:50051")
        .await.expect("cannot connect.");

    HttpServer::new(move || {
        let cors_conf = Cors::default()
            .allowed_methods(["GET"])
            .allow_any_origin()
            .max_age(3600);

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors_conf)
            .wrap(CageAuth::new(verification_server_conf.clone(), validator))
            .app_data(Data::new(pool.clone()))
            .default_service(web::get().to(index))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

async fn validator(mut client: CageApiClient<Channel>, req: ServiceRequest, credentials: Credentials) -> CageMiddlewareTaskResult {
    // Todo: Replace tonic(grpc) connection for validator task
    println!("{}", credentials.get_token());
    Ok(req)
}