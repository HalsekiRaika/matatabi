use std::net::ToSocketAddrs;

use sqlx::{Postgres, Transaction};
use tonic::{Request, Response, Status};
use yarn::yarn_api_server::{YarnApiServer, YarnApi};
use yarn::{Affiliation, Channel, VTuber, Live, TaskResult};

use crate::database::models::id_object::AffiliationId;
use crate::database::models::affiliation_object::Affiliations;
use crate::logger::Logger;

pub mod yarn { tonic::include_proto!("yarn"); }

#[derive(Debug)]
pub struct YarnUpdater {
    pool: sqlx::Pool<Postgres>
}

// Todo: Sending one request per update task is not very efficient, so change to Stream.
#[allow(dead_code, unused_variables)]
#[tonic::async_trait]
impl YarnApi for YarnUpdater {
    async fn insert_affiliation(
        &self, request: Request<Affiliation>
    ) -> Result<Response<TaskResult>, Status> {
        let logger = Logger::new(Some("affiliation/yarn"));
        let reply = TaskResult { message: "".to_string() };
        let mut transaction = self.pool.begin()
            .await
            .expect("cannot build transaction.");
        let _ = Affiliations {
            affiliation_id: AffiliationId(request.get_ref().affiliation_id.clone()),
            name: request.into_inner().name
        }.insert(&mut transaction).await;

        Ok(Response::new(reply))
    }

    async fn insert_v_tuber(&self, request: Request<VTuber>) -> Result<Response<TaskResult>, Status> {
        let logger = Logger::new(Some("vtuber/yarn"));
        let reply = TaskResult { message: "".to_string() };
        Ok(Response::new(reply))
    }

    async fn insert_channel(&self, request: Request<Channel>) -> Result<Response<TaskResult>, Status> {
        let logger = Logger::new(Some("channel/yarn"));
        let reply = TaskResult { message: "".to_string() };
        Ok(Response::new(reply))
    }

    async fn insert_live(&self, request: Request<Live>) -> Result<Response<TaskResult>, Status> {
        let logger = Logger::new(Some("live/yarns"));
        let reply = TaskResult { message: "".to_string() };
        Ok(Response::new(reply))
    }
}


pub async fn yarn_grpc_server() {
    let bind_ip = "[::1]:50051".to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    
    //Todo : Write method for server thread.
}