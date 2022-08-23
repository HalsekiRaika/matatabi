use std::collections::vec_deque::VecDeque;
use std::fmt::Display;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use chrono::{DateTime, Local, TimeZone};

use sqlx::Postgres;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use proto::salmon_api_server::{SalmonApiServer, SalmonApi};
use proto::{Affiliation, Channel, Liver, Video, TaskResult, Void};

use crate::database::models::{Accessor, Fetch};
use crate::database::models::affiliation_object::AffiliationObject;
use crate::database::models::livers_object::LiverObject;
use crate::database::models::channel_object::{ChannelObject, ChannelObjectBuilder};
use crate::database::models::upcoming_object::{VideoObject, InitVideoObject};
use crate::database::models::id_object::{ChannelId, LiverId, VideoId};

#[allow(clippy::all, rustdoc::all)]
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
type SalmonResponseStream<T> = Pin<Box<dyn futures::Stream<Item = Result<T, Status>> + Send>>;

#[tonic::async_trait]
impl SalmonApi for SalmonAutoCollector {
    async fn insert_video(&self, req: Request<Streaming<Video>>) -> SalmonResult<TaskResult> {
        self.collect::<Video, VideoObject>(req).await
    }

    async fn insert_channel(&self, req: Request<Streaming<Channel>>) -> SalmonResult<TaskResult> {
        self.collect::<Channel, ChannelObject>(req).await
    }

    async fn insert_liver(&self, req: Request<Streaming<Liver>>) -> SalmonResult<TaskResult> {
        self.collect::<Liver, LiverObject>(req).await
    }

    async fn insert_affiliation(&self, req: Request<Streaming<Affiliation>>) -> SalmonResult<TaskResult> {
        self.collect::<Affiliation, AffiliationObject>(req).await
    }

    type FetchAllVideosStream = SalmonResponseStream<Video>;
    async fn fetch_all_videos(&self, _: Request<Void>) -> SalmonResult<Self::FetchAllVideosStream> {
        self.fetch::<VideoObject, Video>().await
    }

    type FetchAllChannelsStream = SalmonResponseStream<Channel>;
    async fn fetch_all_channels(&self, _: Request<Void>) -> SalmonResult<Self::FetchAllChannelsStream> {
        self.fetch::<ChannelObject, Channel>().await
    }

    type FetchAllLiversStream = SalmonResponseStream<Liver>;
    async fn fetch_all_livers(&self, _: Request<Void>) -> SalmonResult<Self::FetchAllLiversStream> {
        self.fetch::<LiverObject, Liver>().await
    }

    type FetchAllAffiliationsStream = SalmonResponseStream<Affiliation>;
    async fn fetch_all_affiliations(&self, _: Request<Void>) -> SalmonResult<Self::FetchAllAffiliationsStream> {
        self.fetch::<AffiliationObject, Affiliation>().await
    }
}

impl SalmonAutoCollector {
    pub async fn collect<R, T>(&self, receive: Request<Streaming<R>>) -> SalmonResult<TaskResult>
        where T: From<R> + Display + Accessor<Item = T>,
              R: DeleteFlag
    {
        use futures::StreamExt;
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

        for (delete_flag, item) in collector_item {
            if item.exists(&mut transaction).await
                .map_err(|e| Status::internal(format!("Failed func exists: {:?}", e)))?{
                if delete_flag {
                    let del = item.delete(&mut transaction).await
                        .map_err(|e| Status::internal(format!("Failed func delete: {:?}", e)))?;
                        tracing::debug!("{:<10} {}", yansi::Paint::magenta("delete"), del)
                } else if !item.compare(&mut transaction).await
                    .map_err(|e| Status::internal(format!("Failed func compare: {:?}", e)))? {
                    let upd = item.update(&mut transaction).await
                        .map_err(|e| Status::internal(format!("Failed func update: {:?}", e)))?;
                        tracing::debug!("{:<10} ┌ {}", yansi::Paint::yellow("update old"), upd.0);
                        tracing::debug!("{:<10} ┕ {}", yansi::Paint::yellow("update new"), upd.1);
                } else {
                    tracing::debug!("{:<10} {}", yansi::Paint::blue("ignored"), item)
                }
            } else if !delete_flag {
                let ins = item.insert(&mut transaction).await
                    .map_err(|e| Status::internal(format!("Failed func insert: {:?}", e)))?;
                tracing::debug!("{:<10} {}", yansi::Paint::cyan("insert"), ins);
            } else {
                tracing::debug!("{:<10} {}", yansi::Paint::blue("ignored"), item)
            }
        }

        transaction.commit().await
            .map_err(|e| Status::internal(format!("Failed to commit: {:?}", e)))?;

        tracing::info!("transaction elapsed {}ms", dur_now.elapsed().as_millis());
        Ok(Response::new(TaskResult { message: "".to_string() }))
    }

    pub async fn fetch<D, G>(&self) -> SalmonResult<SalmonResponseStream<G>>
        where D: From<G> + Send + 'static + Display + Fetch<Item = D>,
              G: From<D> + Send + 'static
    {
        use tokio_stream::StreamExt;
        let mut transaction = self.pool.begin().await
            .map_err(|e| Status::failed_precondition(format!("Failed to begin build transaction: {:?}", e)))?;
        let db_item = D::fetch_all(&mut transaction).await
            .map_err(|e| Status::internal(format!("Failed fetch: {:?}", e)))?
            .into_iter()
            .map(G::from)
            .collect::<Vec<_>>();
        let mut stream = Box::pin(tokio_stream::iter(db_item).throttle(std::time::Duration::from_millis(20)));

        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {},
                    Err(_) => { break; }
                }
            }
        });

        let res_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(res_stream) as SalmonResponseStream<G>))
    }
}

impl From<Affiliation> for AffiliationObject {
    fn from(data: Affiliation) -> Self {
        AffiliationObject::new(data.affiliation_id, data.name)
    }
}

impl From<AffiliationObject> for Affiliation {
    fn from(obj: AffiliationObject) -> Self {
        Self { 
            affiliation_id: obj.affiliation_id().into(),
            name: obj.name().to_owned(),
            delete: false
        }
    }
}

impl From<Liver> for LiverObject {
    fn from(data: Liver) -> Self {
        LiverObject::new(data.liver_id, data.affiliation_id, data.name, data.localized_name)
    }
}

impl From<LiverObject> for Liver {
    fn from(obj: LiverObject) -> Self {
        Self {
            liver_id: obj.liver_id().into(),
            name: obj.name().to_owned(),
            localized_name: obj.localized_name().to_owned(),
            affiliation_id: obj.affiliation_id().map(Into::into),
            delete: false
        }
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

impl From<ChannelObject> for Channel {
    fn from(obj: ChannelObject) -> Self {
        Self {
            channel_id: obj.channel_id().to_owned().into(),
            liver_id: obj.liver_id().map(Into::into),
            logo_url: obj.logo_url().to_owned(),
            published_at: Some(::prost_types::Timestamp::from(std::time::SystemTime::from(obj.published_at()))),
            description: obj.description().to_owned(),
            delete: false
        }
    }
}

impl From<Video> for VideoObject {
    fn from(data: Video) -> Self {
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

impl From<VideoObject> for Video {
    fn from(obj: VideoObject) -> Self {
        Self {
            video_id: obj.video_id().to_owned().into(),
            channel_id: obj.channel_id().map(|id| id.to_owned().into()),
            title: obj.title().to_owned(),
            description: obj.description().to_owned(),
            published_at: obj.published_at().map(|at| ::prost_types::Timestamp::from(std::time::SystemTime::from(at))),
            updated_at: obj.updated_at().map(|at| ::prost_types::Timestamp::from(std::time::SystemTime::from(at))),
            will_start_at: obj.will_start_at().map(|at| ::prost_types::Timestamp::from(std::time::SystemTime::from(at))),
            started_at: obj.started_at().map(|at| ::prost_types::Timestamp::from(std::time::SystemTime::from(at))),
            delete: false
        }
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

impl DeleteFlag for Video {
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