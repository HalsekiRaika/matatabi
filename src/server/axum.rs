use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;
use sqlx::{Pool, Postgres};
use crate::routing;

pub async fn run_webapi_server(connection_instance: Pool<Postgres>) {
    let app = Router::new()
        .route("/", get(routing::version))
        .layer(axum::Extension(connection_instance));

    let bind_address = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", bind_address);
    axum::Server::bind(&bind_address)
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Cannot startup webapi server!"))
}
