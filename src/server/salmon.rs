use std::collections::vec_deque::VecDeque;
use std::fmt::Display;
use std::net::ToSocketAddrs;
use chrono::{DateTime, Local, TimeZone};
use futures::StreamExt;

use sqlx::Postgres;
use tokio::time::Instant;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use proto::salmon_api_server::{SalmonApiServer, SalmonApi};
use proto::{Affiliation, Channel, Liver, Live, TaskResult};

use crate::database::models::Transact;
use crate::database::models::affiliation_object::Affiliations;
use crate::database::models::channel_object::{Channels, ChannelsBuilder};
use crate::database::models::id_object::{ChannelId, LiverId, VideoId};
use crate::database::models::livers_object::Livers;
use crate::database::models::upcoming_object::{Lives, InitLives};
use crate::database::models::update_signature::{LatestEq, Signed, UpdateSignature, Version};

#[allow(clippy::module_inception)]
mod proto { tonic::include_proto!("salmon"); }

#[derive(Debug)]
pub struct SalmonAutoCollector {
    pool: sqlx::Pool<Postgres>
}

impl SalmonAutoCollector {
    fn new(connection_pool: sqlx::Pool<Postgres>) -> Self {
        Self { pool: connection_pool }
    }
}

type SalmonResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl SalmonApi for SalmonAutoCollector {
    async fn insert_req_live(&self, req: Request<Streaming<Live>>) -> SalmonResult<TaskResult> {
        self.collect::<Live, Lives>(req).await
    }

    async fn insert_req_channel(&self, req: Request<Streaming<Channel>>) -> SalmonResult<TaskResult> {
        self.collect::<Channel, Channels>(req).await
    }

    async fn insert_req_v_tuber(&self, req: Request<Streaming<Liver>>) -> SalmonResult<TaskResult> {
        self.collect::<Liver, Livers>(req).await
    }

    async fn insert_req_affiliation(&self, req: Request<Streaming<Affiliation>>) -> SalmonResult<TaskResult> {
        self.collect::<Affiliation, Affiliations>(req).await
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

impl From<Affiliation> for Affiliations {
    fn from(data: Affiliation) -> Affiliations {
        Affiliations::new(data.affiliation_id, data.name, data.override_at)
    }
}

impl From<Liver> for Livers {
    fn from(data: Liver) -> Self {
        Livers::new(data.liver_id, data.affiliation_id, data.name, data.localized_name, data.override_at)
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
        InitLives {
            video_id: VideoId(data.video_id),
            channel_id: data.channel_id.map(ChannelId),
            title: data.title,
            description: data.description,
            published_at: data.published_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            updated_at: data.updated_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            will_start_at: data.will_start_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            started_at: data.started_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            thumbnail_url: format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", cloned),
            update_signatures: UpdateSignature(data.override_at),
            ..Default::default()
        }.build()
    }
}

pub async fn run_salmon(pool: sqlx::Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let bind_ip = "[::1]:50051".to_socket_addrs()
        .unwrap().next()
        .unwrap();
    let server = SalmonAutoCollector::new(pool);
    tokio::spawn(async move {
        tracing::debug!("listening salmon autocollector from {}", bind_ip);
        Server::builder()
            .add_service(SalmonApiServer::new(server))
            .serve(bind_ip)
            .await
            .expect("Server failed to start...")
    });

    Ok(())
}