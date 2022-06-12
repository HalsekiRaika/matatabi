use std::net::ToSocketAddrs;
use serde::Serialize;
use chrono::{DateTime, Local, TimeZone};
use futures::StreamExt;

use sqlx::Postgres;
use tokio::time::Instant;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use yansi::Paint;
use salmon::salmon_api_server::{SalmonApiServer, SalmonApi};
use salmon::{Affiliation, Channel, Liver, Live, TaskResult};

use crate::database::models::{Printable, Updatable, Transactable};
use crate::database::models::affiliation_object::Affiliations;
use crate::database::models::channel_object::{Channels, ChannelsBuilder};
use crate::database::models::id_object::{ChannelId, LiverId, VideoId};
use crate::database::models::livers_object::Livers;
use crate::database::models::upcoming_object::{Lives, LivesBuilder};
use crate::database::models::update_signature::UpdateSignature;
use crate::logger::Logger;

#[allow(clippy::module_inception)]
pub mod salmon { tonic::include_proto!("salmon"); }

#[derive(Debug)]
pub struct SalmonUpdater {
    pool: sqlx::Pool<Postgres>
}

impl SalmonUpdater {
    fn new(connection_pool: sqlx::Pool<Postgres>) -> Self {
        Self { pool: connection_pool }
    }
}

type YarnResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl SalmonApi for SalmonUpdater {
    async fn insert_req_live(&self, req: Request<Streaming<Live>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Live, Lives>(req).await
    }

    async fn insert_req_channel(&self, req: Request<Streaming<Channel>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Channel, Channels>(req).await
    }

    async fn insert_req_v_tuber(&self, req: Request<Streaming<Liver>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Liver, Livers>(req).await
    }

    async fn insert_req_affiliation(&self, req: Request<Streaming<Affiliation>>) -> YarnResult<TaskResult> {
        self.transition_insert::<Affiliation, Affiliations>(req).await
    }
}

impl SalmonAutoCollector {
    #[deprecated]
    async fn transition_insert<R, T>(&self, req: Request<Streaming<R>>) -> SalmonResult<TaskResult>
      where T: Transactable<T> + Printable + Updatable + From<R> {
        let logger = Logger::new(Some("Salmon"));
        logger.info(&format!("Salmon WebAPI Grpc connected from: {}", req.remote_addr().unwrap()));

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
                    Err(reason) => return { println!("{:?}", reason); Err(Status::internal("Failed to data insert.")) }
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

        let logger = Logger::new(Some("Salmon"));
        let transaction_elapsed = dur_now.elapsed().as_millis();
        result.elapsed(receive_elapsed, transaction_elapsed);
        logger.info(&format!("Transaction elapsed {}ms", &transaction_elapsed));
        Ok(Response::new(TaskResult { message: result.message() }))
    }
}

impl SalmonAutoCollector {
    pub async fn collect<R, T>(&self, receive: Request<Streaming<R>>) -> SalmonResult<TaskResult>
        where T: From<R> + Display + Transact<TransactItem = T> + Version + LatestEq<ComparisonItem = T> + Signed
    {
        const ZERO_VER: UpdateSignature = UpdateSignature(0);
        let dur_now = Instant::now();
        let collector_item = receive.into_inner()
            .map(Result::unwrap)
            .map(T::from)
            .inspect(|transact_item| tracing::debug!("{:<10} {}", yansi::Paint::green("receive"), transact_item))
            .collect::<VecDeque<T>>()
            .await;
        tracing::info!("received data: {}ms", dur_now.elapsed().as_millis());

        let dur_now = Instant::now();
        let mut transaction = self.pool.begin().await
            .map_err(|e| Status::failed_precondition(format!("Failed to begin build transaction: {:?}", e)))?;

        for item in collector_item {
            if !item.exists(&mut transaction).await
                .map_err(|e| Status::internal(format!("insert: {:?}", e)))? {
                let ins = item.apply(UpdateSignature::default()).insert(&mut transaction).await
                    .map_err(|e| Status::internal(format!("{:?}", e)))?;
                tracing::debug!("{:<10} {}", yansi::Paint::cyan("insert"), ins);
            } else if !item.irregular_sign() && item.version_compare(item.sign(&mut transaction).await
                .map_err(|e| Status::internal(format!("version_compare: {:?}", e)))?) {
                let upd = item.update(&mut transaction).await
                    .map_err(|e| Status::internal(format!("update: {:?}", e)))?;
                tracing::debug!("{:<10} old: {}", yansi::Paint::yellow("update"), upd.0);
                tracing::debug!("{:<10} new: {}", yansi::Paint::yellow("update"), upd.1);
            } else if item.version() < ZERO_VER && item.exists(&mut transaction).await
                .map_err(|e| Status::internal(format!("delete: {:?}", e)))? {
                let del = item.delete(&mut transaction).await
                    .map_err(|e| Status::internal(format!("{:?}", e)))?;
                tracing::debug!("{:<10} {}", yansi::Paint::magenta("delete"), del)
            }
        }

        transaction.commit().await
            .map_err(|e| Status::internal(format!("Failed to commit: {:?}", e)))?;

        tracing::info!("transaction elapsed {}ms", dur_now.elapsed().as_millis());
        Ok(Response::new(TaskResult { message: "".to_string() }))
    }
}

#[deprecated]
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
        serde_json::to_string(self).unwrap_or_else(|_| "result is not available.".to_string())
    }
}

impl From<Affiliation> for Affiliations {
    fn from(data: Affiliation) -> Affiliations {
        Affiliations::new(data.affiliation_id, data.name, data.override_at)
    }
}

impl From<Liver> for Livers {
    fn from(data: Liver) -> Self {
        Livers::new(data.liver_id, data.affiliation_id, data.name, data.override_at)
    }
}

impl From<Channel> for Channels {
    fn from(data: Channel) -> Self {
        let timestamp = if let Some(stamp) = data.published_at { (stamp.seconds, stamp.nanos as u32) } else { (0, 0) };
        let date: DateTime<Local> = Local.timestamp(timestamp.0, timestamp.1);
        ChannelsBuilder {
            channel_id: ChannelId(data.channel_id),
            liver_id: data.liver_id.map(LiverId),
            logo_url: data.logo_url,
            published_at: date,
            description: data.description,
            update_signatures: UpdateSignature(data.override_at),
            ..Default::default()
        }.build()
    }
}

impl From<Live> for Lives {
    fn from(data: Live) -> Self {
        let cloned = data.video_id.clone();
        LivesBuilder {
            video_id: VideoId(data.video_id),
            channel_id: data.channel_id.map(ChannelId),
            title: data.title,
            description: data.description,
            published_at: data.published_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            updated_at: data.updated_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            will_start_at: data.will_start_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            started_at: data.started_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            thumbnail_url: format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", cloned),
            update_signature: UpdateSignature(data.override_at),
            ..Default::default()
        }.build()
    }
}

pub async fn run_salmon(pool: sqlx::Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new(Some("Salmon"));
    let bind_ip = "[::1]:50051".to_socket_addrs()
        .unwrap().next()
        .unwrap();
    let server = SalmonUpdater::new(pool);
    tokio::spawn(async move {
        logger.info("Starting salmon grpc server!");
        Server::builder()
            .add_service(SalmonApiServer::new(server))
            .serve(bind_ip)
            .await
            .expect("Server failed to start...")
    });

    Ok(())
}