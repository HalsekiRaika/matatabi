use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;
use sqlx::{Pool, Postgres};
use crate::routing;

pub async fn run_webapi_server(connection_instance: Pool<Postgres>) {
    let app = Router::new()
        .route("/", get(routing::version))
        .route("/affiliations", get(routing::get_affiliations))
        .route("/affiliations/:id", get(routing::get_affiliation_from_id))
        .route("/livers", get(routing::get_livers))
        .route("/livers/filtered", get(routing::get_livers_filtered))
        .route("/channels", get(routing::get_channels))
        .route("/upcomings", get(routing::get_upcomings))
        .layer(axum::Extension(connection_instance));

    let bind_address = SocketAddr::from(([127, 0, 0, 1], 4500));
    tracing::debug!("listening on {}", bind_address);
    axum::Server::bind(&bind_address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(exit())
        .await
        .unwrap_or_else(|_| panic!("Cannot startup webapi server!"))
}

async fn exit() {
    let user_interrupt = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install keyboard interrupt.")
    };

    tokio::select! {
        _ = user_interrupt => {}
    }

    tracing::info!("interrupt signal received. shutdown.")
}