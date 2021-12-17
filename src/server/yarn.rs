use std::net::ToSocketAddrs;
use futures::StreamExt;

use sqlx::{Error, Postgres};
use tokio::time::Instant;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use yarn::yarn_api_server::{YarnApiServer, YarnApi};
use yarn::{Affiliation, Channel, VTuber, Live, TaskResult};

use crate::database::models::affiliation_object::Affiliations;
use crate::database::models::{Printable, Updatable};
use crate::database::models::upcoming_object::Lives;
use crate::database::models::update_signature::UpdateSignature;
use crate::database::transactable::Transactable;
use crate::logger::Logger;
use crate::server::yarn::yarn::affiliation::OverrideSign;

pub mod yarn { tonic::include_proto!("yarn"); }

#[derive(Debug)]
pub struct YarnUpdater {
    pool: sqlx::Pool<Postgres>
}

impl YarnUpdater {
    fn new(connection_pool: sqlx::Pool<Postgres>) -> Self {
        Self { pool: connection_pool }
    }
}

type YarnResult<T> = Result<Response<T>, Status>;

// Todo: Sending one request per update task is not very efficient, so change to Stream.
#[tonic::async_trait]
impl YarnApi for YarnUpdater {
    async fn insert_req_live(&self, req: Request<Streaming<Live>>) -> YarnResult<TaskResult> {
        let logger = Logger::new(Some("yarn / live"));
        logger.info(&format!("Yarn Updater connected from: {}", req.remote_addr().unwrap()));

        let mut transaction = self.pool.begin().await
            .expect("cannot begin transaction");

        let mut update_data_stream = req.into_inner();
        let mut receive_count = 0;
        let dur_now = Instant::now();

        while let Some(live_data) = update_data_stream.next().await {
            let live_data = live_data?;
            logger.info(&format!("Receive Data : {}", live_data.video_id.clone()));

            receive_count += 1;

            Lives::from(live_data).insert(&mut transaction).await.unwrap();
        }

        let elapsed = dur_now.elapsed().as_millis();
        let response = TaskResult {
            message: format!("Data Received and insert database. item: {}/ elapsed: {}", &receive_count, &elapsed)
        };

        Ok(Response::new(response))
    }

    async fn insert_req_channel(&self, req: Request<Streaming<Channel>>) -> YarnResult<TaskResult> {
        Err(Status::unimplemented("client task is not implemented."))
    }

    async fn insert_req_affiliation(&self, req: Request<Streaming<Affiliation>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Affiliation, Affiliations>(req).await
    }

    async fn insert_req_v_tuber(&self, req: Request<Streaming<VTuber>>) -> YarnResult<TaskResult> {
        Err(Status::unimplemented("client task is not implemented."))
    }
}

impl YarnUpdater {
    async fn transition_insert<R, T>(&self, req: Request<Streaming<R>>) -> YarnResult<TaskResult>
      where T: Transactable + Printable + Updatable + From<R> {
        let logger = Logger::new(Some("Yarn"));
        logger.info(&format!("Yarn Updater connected from: {}", req.remote_addr().unwrap()));

        let mut update_data_stream = req.into_inner();
        let mut insertion: Vec<T> = Vec::new();
        let mut receive_count = 0;
        let dur_now = Instant::now();

        logger.info("+----------+---------------------------+");
        while let Some(receive) = update_data_stream.next().await {
            let receive = receive?;
            let receive = T::from(receive);
            logger.info(&format!("| RECEIVED | {}", receive.get_primary_name()));
            insertion.push(receive);
            receive_count += 1;
        }
        logger.info("+----------+---------------------------+");
        let elapsed = dur_now.elapsed().as_millis();
        let dur_now = Instant::now();

        let response = TaskResult {
            message: format!("Data Received and insert database. item: {} / elapsed: {}ms", &receive_count, &elapsed)
        };

        let mut transaction = self.pool.begin().await.unwrap();
        for item in &insertion {
            if !item.exists(&mut transaction).await.unwrap() {
                item.apply_signature(UpdateSignature::default().as_i64())
                    .insert(&mut transaction)
                    .await;
            } else if !item.is_empty_sign() && item.can_update(&mut transaction).await.unwrap() {
                item.update(&mut transaction)
                    .await;
            }
        }

        transaction.commit().await;
        logger.info("+----------+---------------------------+");
        let elapsed = dur_now.elapsed().as_millis();

        logger.info(&format!("Transaction elapsed {}ms", &elapsed));
        Ok(Response::new(response))
    }
}

impl From<Live> for Lives {
    fn from(data: Live) -> Self {
        Lives::new(data.video_id, data.channel_id, data.title, data.description,
            data.published_at, data.updated_at, Some(data.will_start_at), Some(data.started_at),
            data.thumbnail_url,
        )
    }
}

impl From<Affiliation> for Affiliations {
    fn from(data: Affiliation) -> Affiliations {
        let sign = if let Some(sign) = data.override_sign { match sign {
                OverrideSign::OverrideAt(signature) => { signature }
                OverrideSign::Empty(_) => { 0 }
            }
        } else { 0 };
        Affiliations::new(data.affiliation_id, data.name, sign)
    }
}

pub async fn run_yarn(pool: sqlx::Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new(Some("Yarn"));
    logger.info("Starting yarn grpc update server!");
    let bind_ip = "[::1]:50051".to_socket_addrs()
        .unwrap().next()
        .unwrap();
    let server = YarnUpdater::new(pool);

    Server::builder()
        .add_service(YarnApiServer::new(server))
        .serve(bind_ip)
        .await
        .unwrap();

    Ok(())
}