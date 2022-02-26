use std::net::ToSocketAddrs;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local, TimeZone};
use futures::StreamExt;
use prost_types::Timestamp;

use sqlx::Postgres;
use tokio::time::Instant;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use yansi::Paint;
use yarn::yarn_api_server::{YarnApiServer, YarnApi};
use yarn::{Affiliation, Channel, VTuber, Live, TaskResult};

use crate::database::models::{Printable, Updatable, Transactable};
use crate::database::models::affiliation_object::Affiliations;
use crate::database::models::channel_object::{Channels, ChannelsBuilder};
use crate::database::models::id_object::{ChannelId, LiverId};
use crate::database::models::livers_object::Livers;
use crate::database::models::upcoming_object::Lives;
use crate::database::models::update_signature::UpdateSignature;
use crate::logger::Logger;

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

    async fn insert_req_channel(&self, req: Request<Streaming<Channel>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Channel, Channels>(req).await
    }

    async fn insert_req_affiliation(&self, req: Request<Streaming<Affiliation>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Affiliation, Affiliations>(req).await
    }

    async fn insert_req_v_tuber(&self, req: Request<Streaming<VTuber>>) -> YarnResult<TaskResult> {
        self.transition_insert::<VTuber, Livers>(req).await
    }
}

impl YarnUpdater {
    async fn transition_insert<R, T>(&self, req: Request<Streaming<R>>) -> YarnResult<TaskResult>
      where T: Transactable<T> + Printable + Updatable + From<R> {
        let logger = Logger::new(Some("Yarn"));
        logger.info(&format!("Yarn Updater connected from: {}", req.remote_addr().unwrap()));

        let mut update_data_stream = req.into_inner();
        let mut insertion: Vec<T> = Vec::new();
        let mut result = ResultMsg::default();
        let dur_now = Instant::now();

        while let Some(receive) = update_data_stream.next().await {
            let receive = receive?;
            let receive = T::from(receive);
            logger.info(&format!("[ {:<10} ] {}", Paint::green("RECEIVE"), receive.get_primary_name()));
            insertion.push(receive);
            result.received()
        }
        let receive_elapsed = dur_now.elapsed().as_millis();
        let dur_now = Instant::now();

        let mut transaction = match self.pool.begin().await {
            Ok(transaction) => transaction,
            Err(_) => return Err(Status::failed_precondition("Failed to begin build transaction."))
        };
        let logger = Logger::new(Some("Transaction"));
        for item in &insertion {
            if !item.exists(&mut transaction).await.unwrap() {
                let insert = match item.apply_signature(UpdateSignature::default().as_i64()).insert(&mut transaction).await {
                    Ok(insert) => insert,
                    Err(reason) => return { println!("{}", reason.to_string()); Err(Status::internal("Failed to data insert.")) }
                };
                logger.info(&format!("[ {:<10} ] {} + {}", Paint::cyan("INSERT"), insert.get_secondary_name(), insert.get_signature()));
                result.inserted();
            } else if !item.is_empty_sign() && item.can_update(&mut transaction).await.unwrap_or(false) {
                let (old, update) = match item.update(&mut transaction).await {
                    Ok((old, update)) => (old, update),
                    Err(_) => return Err(Status::internal("Failed to data update."))
                };
                logger.info(&format!("[ {:<10} ] {} : {} > {}", Paint::yellow("UPDATE"),
                    &update.get_secondary_name(), &old.get_primary_name(), &update.get_primary_name()));
                result.updated();
            } else if item.exists(&mut transaction).await.unwrap() && item.get_signature() < 0 {
                let delete = match item.delete(&mut transaction).await {
                    Ok(delete) => delete,
                    Err(_) => return Err(Status::internal("Failed to data delete."))
                };
                logger.caut(&format!("[ {:<10} ] {}", Paint::magenta("DELETE"), delete));
                result.deleted();
            } else {
                result.skipped();
            }
        }

        match transaction.commit().await {
            Ok(_) => (),
            Err(_) => return Err(Status::internal("Failed to commit when inserting data in the database."))
        }

        let logger = Logger::new(Some("Yarn"));
        let transaction_elapsed = dur_now.elapsed().as_millis();
        result.elapsed(receive_elapsed, transaction_elapsed);
        logger.info(&format!("Transaction elapsed {}ms", &transaction_elapsed));
        Ok(Response::new(TaskResult { message: result.message() }))
    }
}

#[derive(Default, Serialize)]
struct ResultMsg {
    receive: u32,
    insert_count: u32,
    update_count: u32,
    delete_count: u32,
    skip_count: u32,
    receive_elapsed: u128,
    transaction_elapsed: u128
}

impl ResultMsg {
    fn received(&mut self) { self.receive += 1; }
    fn inserted(&mut self) { self.insert_count += 1; }
    fn updated(&mut self) { self.update_count += 1; }
    fn deleted(&mut self) { self.delete_count += 1; }
    fn skipped(&mut self) { self.skip_count += 1; }
    fn elapsed(&mut self, receive: u128, transaction: u128) {
        self.receive_elapsed = receive;
        self.transaction_elapsed = transaction;
    }

    fn message(&self) -> String {
        serde_json::to_string(self).unwrap_or("result is not available.".to_string())
    }
}

impl From<Affiliation> for Affiliations {
    fn from(data: Affiliation) -> Affiliations {
        Affiliations::new(data.affiliation_id, data.name, data.override_at)
    }
}

impl From<VTuber> for Livers {
    fn from(data: VTuber) -> Self {
        let id = if let Some(id) = data.affiliation_id { Some(id) } else { None };
        Livers::new(data.v_tuber_id, id, data.name, data.override_at)
    }
}

impl From<Channel> for Channels {
    fn from(data: Channel) -> Self {
        let timestamp = if let Some(stamp) = data.published_at { (stamp.seconds, stamp.nanos as u32) } else { (0, 0) };
        let date: DateTime<Local> = Local.timestamp(timestamp.0, timestamp.1);
        ChannelsBuilder {
            channel_id: ChannelId(data.channel_id),
            liver_id: if let Some(id) = data.v_tuber_id { Some(LiverId(id)) } else { None },
            logo_url: data.logo_url,
            published_at: date,
            description: data.description,
            update_signatures: UpdateSignature(data.override_at),
            ..Default::default()
        }.build()
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