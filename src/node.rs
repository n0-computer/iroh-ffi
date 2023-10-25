use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures::stream::TryStreamExt;
use iroh::{
    bytes::util::runtime::Handle,
    net::key::SecretKey,
    node::{Node, DEFAULT_BIND_ADDR},
    rpc_protocol::{ProviderRequest, ProviderResponse},
};
use quic_rpc::transport::flume::FlumeConnection;

use crate::block_on;
use crate::doc::{AuthorId, Doc, DocTicket, Entry, NamespaceId};
use crate::error::IrohError as Error;
use crate::key::PublicKey;

pub use iroh::rpc_protocol::CounterStats;

/// Information about a direct address.
/// TODO: when refactoring the iroh.node API, give this an impl
#[derive(Debug)]
pub struct DirectAddrInfo(iroh::net::magicsock::DirectAddrInfo);

#[derive(Debug)]
pub struct ConnectionInfo {
    /// The public key of the endpoint.
    pub public_key: Arc<PublicKey>,
    /// Derp region, if available.
    pub derp_region: Option<u16>,
    /// List of addresses at which this node might be reachable, plus any latency information we
    /// have about that address and the last time the address was used.
    pub addrs: Vec<Arc<DirectAddrInfo>>,
    /// The type of connection we have to the peer, either direct or over relay.
    pub conn_type: ConnectionType,
    /// The latency of the `conn_type`.
    pub latency: Option<Duration>,
    /// Duration since the last time this peer was used.
    pub last_used: Option<Duration>,
}

impl From<iroh::net::magic_endpoint::ConnectionInfo> for ConnectionInfo {
    fn from(value: iroh::net::magic_endpoint::ConnectionInfo) -> Self {
        ConnectionInfo {
            public_key: Arc::new(PublicKey(value.public_key)),
            derp_region: value.derp_region,
            addrs: value
                .addrs
                .iter()
                .map(|a| Arc::new(DirectAddrInfo(a.clone())))
                .collect(),
            conn_type: value.conn_type.into(),
            latency: value.latency,
            last_used: value.last_used,
        }
    }
}

#[derive(Debug)]
pub enum ConnectionType {
    Direct { addr: String, port: u16 },
    Relay { port: u16 },
    Mixed { addr: String, port: u16 },
    None,
}

impl From<iroh::net::magicsock::ConnectionType> for ConnectionType {
    fn from(value: iroh::net::magicsock::ConnectionType) -> Self {
        match value {
            iroh::net::magicsock::ConnectionType::Direct(addr) => ConnectionType::Direct {
                addr: addr.ip().to_string(),
                port: addr.port(),
            },
            iroh::net::magicsock::ConnectionType::Mixed(addr, port) => ConnectionType::Mixed {
                addr: addr.ip().to_string(),
                port: port,
            },
            iroh::net::magicsock::ConnectionType::Relay(port) => ConnectionType::Relay { port },
            iroh::net::magicsock::ConnectionType::None => ConnectionType::None,
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum LiveEvent {
    /// A local insertion.
    InsertLocal {
        /// The inserted entry.
        entry: Entry,
    },
    /// Received a remote insert.
    InsertRemote {
        /// The peer that sent us the entry.
        from: PublicKey,
        /// The inserted entry.
        entry: Entry,
        /// If the content is available at the local node
        content_status: ContentStatus,
    },
    /// The content of an entry was downloaded and is now available at the local node
    ContentReady {
        /// The content hash of the newly available entry content
        hash: Hash,
    },
    /// We have a new neighbor in the swarm.
    NeighborUp(PublicKey),
    /// We lost a neighbor in the swarm.
    NeighborDown(PublicKey),
    /// A set-reconciliation sync finished.
    SyncFinished(SyncEvent),
}

pub enum LiveEventType {
    InsertLocal,
    InsertRemote,
    ContentReady,
    NeighborUp,
    NeighborDown,
    SyncFinished,
}

impl LiveEvent {
    pub fn r#type(&self) -> LiveEventType {
        match self {
            Self::InsertLocal { .. } => LiveEventType::InsertLocal,
            Self::InsertRemote { .. } => LiveEventType::InsertRemote,
            Self::ContentReady { .. } => LiveEventType::ContentReady,
            Self::NeighborUp(_) => LiveEventType::NeighborUp,
            Self::NeighborDown(_) => LiveEventType::NeighborDown,
            Self::SyncFinished(_) => LiveEventType::SyncFinished,
        }
    }

    pub fn as_insert_local(&self) -> Arc<Entry> {
        if let Self::InsertLocal { entry } = self {
            Arc::new(entry.clone())
        } else {
            panic!("not an insert local event");
        }
    }

    pub fn as_insert_remote(&self) -> InsertRemoteEvent {
        if let Self::InsertRemote {
            from,
            entry,
            content_status,
        } = self
        {
            InsertRemoteEvent {
                from: Arc::new(from.clone()),
                entry: Arc::new(entry.clone()),
                content_status: content_status.clone(),
            }
        } else {
            panic!("not an insert remote event");
        }
    }

    pub fn as_content_ready(&self) -> Arc<Hash> {
        if let Self::ContentReady { hash } = self {
            Arc::new(hash.clone())
        } else {
            panic!("not an content ready event");
        }
    }

    pub fn as_neighbor_up(&self) -> Arc<PublicKey> {
        if let Self::NeighborUp(key) = self {
            Arc::new(key.clone())
        } else {
            panic!("not an neighbor up event");
        }
    }

    pub fn as_neighbor_down(&self) -> Arc<PublicKey> {
        if let Self::NeighborDown(key) = self {
            Arc::new(key.clone())
        } else {
            panic!("not an neighbor down event");
        }
    }

    pub fn as_sync_finished(&self) -> SyncEvent {
        if let Self::SyncFinished(event) = self {
            event.clone()
        } else {
            panic!("not an sync event event");
        }
    }
}

/// Whether the content status is available on a node.
#[derive(Debug, Clone)]
pub enum ContentStatus {
    /// The content is completely available.
    Complete,
    /// The content is partially available.
    Incomplete,
    /// The content is missing.
    Missing,
}

impl From<iroh::sync::sync::ContentStatus> for ContentStatus {
    fn from(value: iroh::sync::sync::ContentStatus) -> Self {
        match value {
            iroh::sync::sync::ContentStatus::Complete => Self::Complete,
            iroh::sync::sync::ContentStatus::Incomplete => Self::Incomplete,
            iroh::sync::sync::ContentStatus::Missing => Self::Missing,
        }
    }
}

impl From<iroh::sync_engine::LiveEvent> for LiveEvent {
    fn from(value: iroh::sync_engine::LiveEvent) -> Self {
        match value {
            iroh::sync_engine::LiveEvent::InsertLocal { entry } => LiveEvent::InsertLocal {
                entry: entry.into(),
            },
            iroh::sync_engine::LiveEvent::InsertRemote {
                from,
                entry,
                content_status,
            } => LiveEvent::InsertRemote {
                from: from.into(),
                entry: entry.into(),
                content_status: content_status.into(),
            },
            iroh::sync_engine::LiveEvent::ContentReady { hash } => {
                LiveEvent::ContentReady { hash: hash.into() }
            }
            iroh::sync_engine::LiveEvent::NeighborUp(key) => LiveEvent::NeighborUp(key.into()),
            iroh::sync_engine::LiveEvent::NeighborDown(key) => LiveEvent::NeighborDown(key.into()),
            iroh::sync_engine::LiveEvent::SyncFinished(e) => LiveEvent::SyncFinished(e.into()),
        }
    }
}

/// Outcome of a sync operation
#[derive(Debug, Clone)]
pub struct SyncEvent {
    /// Peer we synced with
    pub peer: Arc<PublicKey>,
    /// Origin of the sync exchange
    pub origin: Origin,
    /// Timestamp when the sync finished
    pub finished: SystemTime,
    /// Timestamp when the sync started
    pub started: SystemTime,
    /// Result of the sync operation. `None` if successfull.
    pub result: Option<String>,
}

impl From<iroh::sync_engine::SyncEvent> for SyncEvent {
    fn from(value: iroh::sync_engine::SyncEvent) -> Self {
        SyncEvent {
            peer: Arc::new(value.peer.into()),
            origin: value.origin.into(),
            finished: value.finished,
            started: value.started,
            result: match value.result {
                Ok(_) => None,
                Err(err) => Some(err),
            },
        }
    }
}

// TODO: iroh 0.8.0 release made this struct private. Re-implement when it's made public again
/// Why we started a sync request
// #[derive(Debug, Clone, Copy)]
// pub enum SyncReason {
//     /// Direct join request via API
//     DirectJoin,
//     /// Peer showed up as new neighbor in the gossip swarm
//     NewNeighbor,
// }

// impl From<iroh::sync_engine::SyncReason> for SyncReason {
//     fn from(value: iroh::sync_engine::SyncReason) -> Self {
//         match value {
//             iroh::sync_engine::SyncReason::DirectJoin => Self::DirectJoin,
//             iroh::sync_engine::SyncReason::NewNeighbor => Self::NewNeighbor,
//         }
//     }
// }

/// Why we performed a sync exchange
#[derive(Debug, Clone)]
pub enum Origin {
    /// TODO: in iroh 0.8.0 `SyncReason` is private, until the next release when it can be made
    /// public, use a unit variant
    // Connect {
    //     reason: SyncReason,
    // },
    Connect,
    /// A peer connected to us and we accepted the exchange
    Accept,
}

impl From<iroh::sync_engine::Origin> for Origin {
    fn from(value: iroh::sync_engine::Origin) -> Self {
        match value {
            iroh::sync_engine::Origin::Connect(_) => Self::Connect,
            iroh::sync_engine::Origin::Accept => Self::Accept,
        }
    }
}

#[derive(Debug)]
pub struct InsertRemoteEvent {
    /// The peer that sent us the entry.
    pub from: Arc<PublicKey>,
    /// The inserted entry.
    pub entry: Arc<Entry>,
    /// If the content is available at the local node
    pub content_status: ContentStatus,
}

#[derive(Debug, Clone)]
pub struct Hash(pub(crate) iroh::bytes::Hash);

impl From<iroh::bytes::Hash> for Hash {
    fn from(h: iroh::bytes::Hash) -> Self {
        Hash(h)
    }
}

impl Hash {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl From<Hash> for iroh::bytes::Hash {
    fn from(value: Hash) -> Self {
        value.0
    }
}

pub trait SubscribeCallback: Send + Sync + 'static {
    fn event(&self, event: Arc<LiveEvent>) -> Result<(), Error>;
}

pub struct IrohNode {
    node: Node<iroh::bytes::store::flat::Store>,
    async_runtime: Handle,
    sync_client: iroh::client::Iroh<FlumeConnection<ProviderResponse, ProviderRequest>>,
    #[allow(dead_code)]
    tokio_rt: tokio::runtime::Runtime,
}

impl IrohNode {
    pub fn new(path: String) -> Result<Self, Error> {
        let path = PathBuf::from(path);
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(Error::runtime)?;

        let tpc = tokio_util::task::LocalPoolHandle::new(num_cpus::get());
        let rt = iroh::bytes::util::runtime::Handle::new(tokio_rt.handle().clone(), tpc);

        let rt_inner = rt.clone();
        let node = block_on(&rt, async move {
            // TODO: store and load keypair
            let secret_key = SecretKey::generate();

            let docs_path = path.join("docs.db");
            let docs = iroh::sync::store::fs::Store::new(&docs_path)?;

            // create a bao store for the iroh-bytes blobs
            let blob_path = path.join("blobs");
            tokio::fs::create_dir_all(&blob_path).await?;
            let db = iroh::bytes::store::flat::Store::load(
                &blob_path, &blob_path, &blob_path, &rt_inner,
            )
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

    pub fn doc_list(&self) -> Result<Vec<Arc<NamespaceId>>, Error> {
        block_on(&self.async_runtime, async {
            let docs = self
                .sync_client
                .docs
                .list()
                .await
                .map_err(Error::doc)?
                .map_ok(|n| Arc::new(n.into()))
                .try_collect::<Vec<_>>()
                .await
                .map_err(Error::doc)?;

            Ok(docs)
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

    pub fn blob_list_blobs(&self) -> Result<Vec<Arc<Hash>>, Error> {
        block_on(&self.async_runtime, async {
            let response = self.sync_client.blobs.list().await.map_err(Error::blob)?;

            let hashes: Vec<Arc<Hash>> = response
                .map_ok(|i| Arc::new(Hash(i.hash)))
                .map_err(Error::blob)
                .try_collect()
                .await?;

            Ok(hashes)
        })
    }

    pub fn blob_get(&self, hash: Arc<Hash>) -> Result<Vec<u8>, Error> {
        block_on(&self.async_runtime, async {
            let mut r = self
                .sync_client
                .blobs
                .read(hash.0)
                .await
                .map_err(Error::blob)?;
            let data = r.read_to_bytes().await.map_err(Error::blob)?;
            Ok(data.into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_new() {
        let path = tempfile::tempdir().unwrap();
        let node = IrohNode::new(path.path().to_string_lossy().into_owned()).unwrap();
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
