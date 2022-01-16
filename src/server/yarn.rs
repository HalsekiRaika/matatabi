use std::net::ToSocketAddrs;
use futures::StreamExt;

use sqlx::Postgres;
use tokio::time::Instant;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use yarn::yarn_api_server::{YarnApiServer, YarnApi};
use yarn::{Affiliation, Channel, VTuber, Live, TaskResult};

use crate::database::models::{Printable, Updatable, Transactable};
use crate::database::models::affiliation_object::Affiliations;
use crate::database::models::upcoming_object::Lives;
use crate::database::models::update_signature::UpdateSignature;
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
    async fn insert_req_live(&self, _req: Request<Streaming<Live>>) -> YarnResult<TaskResult> {
        Err(Status::unimplemented("client task is not implemented."))
    }

    async fn insert_req_channel(&self, _req: Request<Streaming<Channel>>) -> YarnResult<TaskResult> {
        Err(Status::unimplemented("client task is not implemented."))
    }

    async fn insert_req_affiliation(&self, req: Request<Streaming<Affiliation>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Affiliation, Affiliations>(req).await
    }

    async fn insert_req_v_tuber(&self, _req: Request<Streaming<VTuber>>) -> YarnResult<TaskResult> {
        Err(Status::unimplemented("client task is not implemented."))
    }
}

impl YarnUpdater {
    async fn transition_insert<R, T>(&self, req: Request<Streaming<R>>) -> YarnResult<TaskResult>
      where T: Transactable<T> + Printable + Updatable + From<R> {
        let logger = Logger::new(Some("Yarn"));
        logger.info(&format!("Yarn Updater connected from: {}", req.remote_addr().unwrap()));

        let mut update_data_stream = req.into_inner();
        let mut insertion: Vec<T> = Vec::new();
        let mut receive_count = 0;
        let dur_now = Instant::now();

        while let Some(receive) = update_data_stream.next().await {
            let receive = receive?;
            let receive = T::from(receive);
            logger.info(&format!("| RECEIVE  | {}", receive.get_primary_name()));
            insertion.push(receive);
            receive_count += 1;
        }
        let elapsed = dur_now.elapsed().as_millis();
        let dur_now = Instant::now();

        let response = TaskResult {
            message: format!("Received. item: {} / elapsed: {}ms", &receive_count, &elapsed)
        };

        let mut transaction = match self.pool.begin().await {
            Ok(transaction) => transaction,
            Err(_) => return Err(Status::failed_precondition("Failed to begin build transaction."))
        };
        let logger = Logger::new(Some("Transaction"));
        for item in &insertion {
            if !item.exists(&mut transaction).await.unwrap() {
                let insert = match item.apply_signature(UpdateSignature::default().as_i64()).insert(&mut transaction).await {
                    Ok(insert) => insert,
                    Err(_) => return Err(Status::internal("Failed to data insert."))
                };
                logger.info(&format!("| INSERT   | {} + {}", insert.get_secondary_name(), insert.get_signature()));
            } else if !item.is_empty_sign() && item.can_update(&mut transaction).await.unwrap() {
                let (old, update) = match item.update(&mut transaction).await {
                    Ok((old, update)) => (old, update),
                    Err(_) => return Err(Status::internal("Failed to data update."))
                };
                logger.info(&format!("| UPDATE   | {} : {} > {}",
                    &update.get_secondary_name(), &old.get_primary_name(), &update.get_primary_name()));
            } else if item.exists(&mut transaction).await.unwrap() && item.get_signature() < 0 {
                let delete = match item.delete(&mut transaction).await {
                    Ok(delete) => delete,
                    Err(_) => return Err(Status::internal("Failed to data delete."))
                };
                logger.caut(&format!("| DELETE   | {}", delete));
            }
        }

        match transaction.commit().await {
            Ok(_) => (),
            Err(_) => return Err(Status::internal("Failed to commit when inserting data in the database."))
        }

        let logger = Logger::new(Some("Yarn"));
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
    let bind_ip = "[::1]:50051".to_socket_addrs()
        .unwrap().next()
        .unwrap();
    let server = YarnUpdater::new(pool);
    tokio::spawn(async move {
        logger.info("Starting yarn grpc update server!");
        Server::builder()
            .add_service(YarnApiServer::new(server))
            .serve(bind_ip)
            .await
            .expect("Server failed to start...")
    });

    Ok(())
}