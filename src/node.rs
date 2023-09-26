use std::{collections::HashMap, sync::Arc};

use futures::{
    stream::{StreamExt, TryStreamExt},
    Future,
};
use iroh::{
    baomap::flat,
    bytes::util::runtime::Handle,
    client::Doc as ClientDoc,
    metrics::try_init_metrics_collection,
    net::key::SecretKey,
    node::{Node, DEFAULT_BIND_ADDR},
    rpc_protocol::{ProviderRequest, ProviderResponse, ShareMode},
};
use quic_rpc::transport::flume::FlumeConnection;

use crate::error::IrohError as Error;

pub use iroh::rpc_protocol::CounterStats;
pub use iroh::sync_engine::LiveStatus;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug)]
pub enum SocketAddr {
    V4 { a: u8, b: u8, c: u8, d: u8 },
    V6 { addr: Vec<u8> },
}

impl From<std::net::SocketAddr> for SocketAddr {
    fn from(value: std::net::SocketAddr) -> Self {
        match value {
            std::net::SocketAddr::V4(addr) => {
                let [a, b, c, d] = addr.ip().octets();
                SocketAddr::V4 { a, b, c, d }
            }
            std::net::SocketAddr::V6(addr) => SocketAddr::V6 {
                addr: addr.ip().octets().to_vec(),
            },
        }
    }
}

#[derive(Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> LevelFilter {
        match level {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off => LevelFilter::OFF,
        }
    }
}

pub fn set_log_level(level: LogLevel) {
    use tracing_subscriber::{fmt, prelude::*, reload};
    let filter: LevelFilter = level.into();
    let (filter, _) = reload::Layer::new(filter);
    let mut layer = fmt::Layer::default();
    layer.set_ansi(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();
}

pub fn start_metrics_collection() -> Result<(), Error> {
    try_init_metrics_collection().map_err(|e| Error::Runtime {
        description: e.to_string(),
    })?;
    Ok(())
}

#[derive(Debug)]
pub struct PublicKey(iroh::net::key::PublicKey);

impl PublicKey {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub id: u64,
    pub public_key: Arc<PublicKey>,
    pub derp_region: Option<u16>,
    pub addrs: Vec<SocketAddr>,
    pub latencies: Vec<Option<f64>>,
    pub conn_type: ConnectionType,
    pub latency: Option<f64>,
}

impl From<iroh::net::magic_endpoint::ConnectionInfo> for ConnectionInfo {
    fn from(value: iroh::net::magic_endpoint::ConnectionInfo) -> Self {
        ConnectionInfo {
            id: value.id as _,
            public_key: Arc::new(PublicKey(value.public_key)),
            derp_region: value.derp_region,
            addrs: value.addrs.iter().map(|(a, _)| (*a).into()).collect(),
            latencies: value
                .addrs
                .iter()
                .map(|(_, l)| l.map(|d| d.as_secs_f64()))
                .collect(),
            conn_type: value.conn_type.into(),
            latency: value.latency.map(|l| l.as_secs_f64()),
        }
    }
}

#[derive(Debug)]
pub enum ConnectionType {
    Direct { addr: SocketAddr },
    Relay { port: u16 },
    None,
}

impl From<iroh::net::magicsock::ConnectionType> for ConnectionType {
    fn from(value: iroh::net::magicsock::ConnectionType) -> Self {
        match value {
            iroh::net::magicsock::ConnectionType::Direct(addr) => {
                ConnectionType::Direct { addr: addr.into() }
            }
            iroh::net::magicsock::ConnectionType::Relay(port) => ConnectionType::Relay { port },
            iroh::net::magicsock::ConnectionType::None => ConnectionType::None,
        }
    }
}

#[derive(Debug)]
pub enum LiveEvent {
    InsertLocal,
    InsertRemote,
    ContentReady,
    SyncFinished,
    NeighborUp,
    NeighborDown,
}

impl From<iroh::sync_engine::LiveEvent> for LiveEvent {
    fn from(value: iroh::sync_engine::LiveEvent) -> Self {
        match value {
            iroh::sync_engine::LiveEvent::InsertLocal { .. } => Self::InsertLocal,
            iroh::sync_engine::LiveEvent::InsertRemote { .. } => Self::InsertRemote,
            iroh::sync_engine::LiveEvent::ContentReady { .. } => Self::ContentReady,
            iroh::sync_engine::LiveEvent::SyncFinished { .. } => Self::SyncFinished,
            iroh::sync_engine::LiveEvent::NeighborUp { .. } => Self::NeighborUp,
            iroh::sync_engine::LiveEvent::NeighborDown { .. } => Self::NeighborDown,
        }
    }
}

pub struct Hash(iroh::bytes::Hash);

impl Hash {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

pub struct Entry(iroh::sync::sync::Entry);

impl Entry {
    pub fn author(&self) -> Arc<AuthorId> {
        Arc::new(AuthorId(self.0.id().author()))
    }

    pub fn key(&self) -> Vec<u8> {
        self.0.id().key().to_vec()
    }

    pub fn hash(&self) -> Arc<Hash> {
        Arc::new(Hash(self.0.content_hash()))
    }

    pub fn namespace(&self) -> Arc<NamespaceId> {
        Arc::new(NamespaceId(self.0.id().namespace()))
    }
}

pub struct Doc {
    inner: ClientDoc<FlumeConnection<ProviderResponse, ProviderRequest>>,
    rt: Handle,
}

impl Doc {
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    pub fn keys(&self) -> Result<Vec<Arc<Entry>>, Error> {
        let latest = block_on(&self.rt, async {
            let get_result = self
                .inner
                .get_many(iroh::sync::store::GetFilter::All)
                .await?;
            get_result
                .map_ok(|e| Arc::new(Entry(e)))
                .try_collect::<Vec<_>>()
                .await
        })
        .map_err(Error::doc)?;
        Ok(latest)
    }

    pub fn share_write(&self) -> Result<Arc<DocTicket>, Error> {
        block_on(&self.rt, async {
            let ticket = self
                .inner
                .share(ShareMode::Write)
                .await
                .map_err(Error::doc)?;

            Ok(Arc::new(DocTicket(ticket)))
        })
    }

    pub fn share_read(&self) -> Result<Arc<DocTicket>, Error> {
        block_on(&self.rt, async {
            let ticket = self
                .inner
                .share(ShareMode::Read)
                .await
                .map_err(Error::doc)?;

            Ok(Arc::new(DocTicket(ticket)))
        })
    }

    pub fn set_bytes(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Arc<Hash>, Error> {
        block_on(&self.rt, async {
            let hash = self
                .inner
                .set_bytes(author_id.0.clone(), key, value)
                .await
                .map_err(Error::doc)?;
            Ok(Arc::new(Hash(hash)))
        })
    }

    pub fn get_content_bytes(&self, entry: Arc<Entry>) -> Result<Vec<u8>, Error> {
        block_on(&self.rt, async {
            let content = self
                .inner
                .read_to_bytes(&entry.0)
                .await
                .map_err(Error::doc)?;

            Ok(content.to_vec())
        })
    }

    pub fn stop_sync(&self) -> Result<(), Error> {
        block_on(&self.rt, async {
            self.inner.stop_sync().await.map_err(Error::doc)?;
            Ok(())
        })
    }

    pub fn status(&self) -> Result<LiveStatus, Error> {
        block_on(&self.rt, async {
            let status = self.inner.status().await.map_err(Error::doc)?;
            Ok(status)
        })
    }

    pub fn subscribe(&self, cb: Box<dyn SubscribeCallback>) -> Result<(), Error> {
        let client = self.inner.clone();
        self.rt.main().spawn(async move {
            let mut sub = client.subscribe().await.unwrap();
            while let Some(event) = sub.next().await {
                match event {
                    Ok(event) => {
                        if let Err(err) = cb.event(event.into()) {
                            println!("cb error: {:?}", err);
                        }
                    }
                    Err(err) => {
                        println!("rpc error: {:?}", err);
                    }
                }
            }
        });

        Ok(())
    }
}

pub trait SubscribeCallback: Send + Sync + 'static {
    fn event(&self, event: LiveEvent) -> Result<(), Error>;
}

pub struct AuthorId(iroh::sync::sync::AuthorId);

impl AuthorId {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct NamespaceId(iroh::sync::sync::NamespaceId);

impl NamespaceId {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub struct DocTicket(iroh::rpc_protocol::DocTicket);

impl DocTicket {
    pub fn from_string(content: String) -> Result<Self, Error> {
        let ticket = content
            .parse::<iroh::rpc_protocol::DocTicket>()
            .map_err(Error::doc_ticket)?;
        Ok(DocTicket(ticket))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct IrohNode {
    node: Node<flat::Store, iroh::sync::store::fs::Store>,
    async_runtime: Handle,
    sync_client: iroh::client::Iroh<FlumeConnection<ProviderResponse, ProviderRequest>>,
    #[allow(dead_code)]
    tokio_rt: tokio::runtime::Runtime,
}

impl IrohNode {
    pub fn new() -> Result<Self, Error> {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(Error::runtime)?;

        let tpc = tokio_util::task::LocalPoolHandle::new(num_cpus::get());
        let rt = iroh::bytes::util::runtime::Handle::new(tokio_rt.handle().clone(), tpc);

        // TODO: pass in path
        let path = tempfile::tempdir().map_err(Error::node_create)?.into_path();

        // TODO: store and load keypair
        let secret_key = SecretKey::generate();

        let rt_inner = rt.clone();
        let node = block_on(&rt, async move {
            let docs_path = path.join("docs.db");
            let docs = iroh::sync::store::fs::Store::new(&docs_path)?;

            // create a bao store for the iroh-bytes blobs
            let blob_path = path.join("blobs");
            tokio::fs::create_dir_all(&blob_path).await?;
            let db = iroh::baomap::flat::Store::load(&blob_path, &blob_path, &blob_path, &rt_inner)
                .await?;

            Node::builder(db, docs)
                .bind_addr(DEFAULT_BIND_ADDR.into())
                .secret_key(secret_key)
                .runtime(&rt_inner)
                .spawn()
                .await
        })
        .map_err(Error::node_create)?;

        let sync_client = node.client();

        Ok(IrohNode {
            node,
            async_runtime: rt,
            sync_client,
            tokio_rt,
        })
    }

    pub fn node_id(&self) -> String {
        self.node.peer_id().to_string()
    }

    pub fn doc_new(&self) -> Result<Arc<Doc>, Error> {
        block_on(&self.async_runtime, async {
            let doc = self.sync_client.docs.create().await.map_err(Error::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.async_runtime.clone(),
            }))
        })
    }

    pub fn author_new(&self) -> Result<Arc<AuthorId>, Error> {
        block_on(&self.async_runtime, async {
            let author = self
                .sync_client
                .authors
                .create()
                .await
                .map_err(Error::author)?;

            Ok(Arc::new(AuthorId(author)))
        })
    }

    pub fn author_list(&self) -> Result<Vec<Arc<AuthorId>>, Error> {
        block_on(&self.async_runtime, async {
            let authors = self
                .sync_client
                .authors
                .list()
                .await
                .map_err(Error::author)?
                .map_ok(|id| Arc::new(AuthorId(id)))
                .try_collect::<Vec<_>>()
                .await
                .map_err(Error::author)?;
            Ok(authors)
        })
    }

    pub fn doc_join(&self, ticket: Arc<DocTicket>) -> Result<Arc<Doc>, Error> {
        block_on(&self.async_runtime, async {
            let doc = self
                .sync_client
                .docs
                .import(ticket.0.clone())
                .await
                .map_err(Error::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.async_runtime.clone(),
            }))
        })
    }

    pub fn stats(&self) -> Result<HashMap<String, CounterStats>, Error> {
        block_on(&self.async_runtime, async {
            let stats = self.sync_client.node.stats().await.map_err(Error::doc)?;
            Ok(stats)
        })
    }

    pub fn connections(&self) -> Result<Vec<ConnectionInfo>, Error> {
        block_on(&self.async_runtime, async {
            let infos = self
                .sync_client
                .node
                .connections()
                .await
                .map_err(Error::connection)?
                .map_ok(|info| info.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(Error::connection)?;
            Ok(infos)
        })
    }

    pub fn connection_info(
        &self,
        node_id: Arc<PublicKey>,
    ) -> Result<Option<ConnectionInfo>, Error> {
        block_on(&self.async_runtime, async {
            let info = self
                .sync_client
                .node
                .connection_info(node_id.as_ref().0)
                .await
                .map(|i| i.map(|i| i.into()))
                .map_err(Error::connection)?;
            Ok(info)
        })
    }
}

fn block_on<F: Future<Output = T>, T>(rt: &Handle, fut: F) -> T {
    tokio::task::block_in_place(move || match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(fut),
        Err(_) => rt.main().block_on(fut),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_new() {
        let node = IrohNode::new().unwrap();
        let node_id = node.node_id();
        println!("id: {}", node_id);
        let doc = node.doc_new().unwrap();
        let doc_id = doc.id();
        println!("doc_id: {}", doc_id);

        let doc_ticket = doc.share_write().unwrap();
        let doc_ticket_string = doc_ticket.to_string();
        let dock_ticket_back = DocTicket::from_string(doc_ticket_string.clone()).unwrap();
        assert_eq!(
            doc_ticket.0.to_bytes().unwrap(),
            dock_ticket_back.0.to_bytes().unwrap()
        );
        println!("doc_ticket: {}", doc_ticket_string);
        node.doc_join(doc_ticket).unwrap();
    }
}
