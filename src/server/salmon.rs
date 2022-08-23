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

use crate::database::models::Accessor;
use crate::database::models::affiliation_object::AffiliationObject;
use crate::database::models::livers_object::LiverObject;
use crate::database::models::channel_object::{ChannelObject, ChannelObjectBuilder};
use crate::database::models::upcoming_object::{VideoObject, InitVideoObject};
use crate::database::models::id_object::{ChannelId, LiverId, VideoId};

#[allow(clippy::all)]
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
        self.collect::<Live, VideoObject>(req).await
    }

    async fn insert_req_channel(&self, req: Request<Streaming<Channel>>) -> SalmonResult<TaskResult> {
        self.collect::<Channel, ChannelObject>(req).await
    }

    async fn insert_req_v_tuber(&self, req: Request<Streaming<Liver>>) -> SalmonResult<TaskResult> {
        self.collect::<Liver, LiverObject>(req).await
    }

    async fn insert_req_affiliation(&self, req: Request<Streaming<Affiliation>>) -> SalmonResult<TaskResult> {
        self.collect::<Affiliation, AffiliationObject>(req).await
    }
}

impl SalmonAutoCollector {
    pub async fn collect<R, T>(&self, receive: Request<Streaming<R>>) -> SalmonResult<TaskResult>
        where T: From<R> + Display + Accessor<Item = T>,
              R: DeleteFlag
    {
        let dur_now = Instant::now();
        let collector_item = receive.into_inner()
            .map(Result::unwrap)    
            .map(|rec| (rec.flagged(), T::from(rec)))
            .inspect(|(_, transact_item)| tracing::debug!("{:<10} {}", yansi::Paint::green("receive"), transact_item))
            .collect::<VecDeque<(bool, T)>>()
            .await;
        tracing::info!("received data: {}ms", dur_now.elapsed().as_millis());

        let dur_now = Instant::now();
        let mut transaction = self.pool.begin().await
            .map_err(|e| Status::failed_precondition(format!("Failed to begin build transaction: {:?}", e)))?;

        for item in collector_item {
            // Todo: implement new logic
        }

        transaction.commit().await
            .map_err(|e| Status::internal(format!("Failed to commit: {:?}", e)))?;

        tracing::info!("transaction elapsed {}ms", dur_now.elapsed().as_millis());
        Ok(Response::new(TaskResult { message: "".to_string() }))
    }
}

impl From<Affiliation> for AffiliationObject {
    fn from(data: Affiliation) -> Self {
        AffiliationObject::new(data.affiliation_id, data.name)
    }
}

impl From<Liver> for LiverObject {
    fn from(data: Liver) -> Self {
        LiverObject::new(data.liver_id, data.affiliation_id, data.name, data.localized_name)
    }
}

impl From<Channel> for ChannelObject {
    fn from(data: Channel) -> Self {
        let timestamp = if let Some(stamp) = data.published_at { (stamp.seconds, stamp.nanos as u32) } else { (0, 0) };
        let date: DateTime<Local> = Local.timestamp(timestamp.0, timestamp.1);
        ChannelObjectBuilder {
            channel_id: ChannelId::new(data.channel_id),
            liver_id: data.liver_id.map(LiverId::new),
            logo_url: data.logo_url,
            published_at: date,
            description: data.description,
            ..Default::default()
        }.build()
    }
}

impl From<Live> for VideoObject {
    fn from(data: Live) -> Self {
        let cloned = data.video_id.clone();
        InitVideoObject {
            video_id: VideoId::new(data.video_id.clone()),
            channel_id: data.channel_id.map(ChannelId::new),
            title: data.title,
            description: data.description,
            published_at: data.published_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            updated_at: data.updated_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            will_start_at: data.will_start_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            started_at: data.started_at.map(|stamp| Local.timestamp(stamp.seconds, stamp.nanos as u32)),
            thumbnail_url: format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", cloned),
            ..Default::default()
        }.build()
    }
}

pub trait DeleteFlag {
    fn flagged(&self) -> bool;
}

impl DeleteFlag for Affiliation {
    fn flagged(&self) -> bool {
        self.delete
    }
}

impl DeleteFlag for Liver {
    fn flagged(&self) -> bool {
        self.delete
    }
}

impl DeleteFlag for Channel {
    fn flagged(&self) -> bool {
        self.delete
    }
}

impl DeleteFlag for Live {
    fn flagged(&self) -> bool {
        self.delete
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