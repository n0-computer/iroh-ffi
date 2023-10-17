use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

use futures::{StreamExt, TryStreamExt};
use iroh::{
    bytes::util::runtime::Handle,
    client::Doc as ClientDoc,
    rpc_protocol::{ProviderRequest, ProviderResponse},
};

use quic_rpc::transport::flume::FlumeConnection;

use crate::{block_on, Hash, IrohError, PublicKey, SocketAddr, SocketAddrType};

pub use iroh::sync::CapabilityKind;

/// A representation of a mutable, synchronizable key-value store.
pub struct Doc {
    pub(crate) inner: ClientDoc<FlumeConnection<ProviderResponse, ProviderRequest>>,
    pub(crate) rt: Handle,
}

impl Doc {
    /// Get the document id of this doc.
    pub fn id(&self) -> Arc<NamespaceId> {
        Arc::new(self.inner.id().into())
    }

    /// Close the document.
    pub fn close(&self) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner.close().await.map_err(IrohError::doc)
        })
    }

    /// Set the content of a key to a byte array.
    pub fn set_bytes(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Arc<Hash>, IrohError> {
        block_on(&self.rt, async {
            let hash = self
                .inner
                .set_bytes(author_id.0, key, value)
                .await
                .map_err(IrohError::doc)?;
            Ok(Arc::new(Hash(hash)))
        })
    }

    pub fn set_file_bytes(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        path: String,
    ) -> Result<Arc<Hash>, IrohError> {
        block_on(&self.rt, async {
            // read file from path
            let value = tokio::fs::read(path).await.map_err(IrohError::doc)?;
            let hash = self
                .inner
                .set_bytes(author_id.0, key, value)
                .await
                .map_err(IrohError::doc)?;
            Ok(Arc::new(Hash(hash)))
        })
    }

    pub fn get_content_bytes(&self, entry: Arc<Entry>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.rt, async {
            let content = self
                .inner
                .read_to_bytes(&entry.0)
                .await
                .map_err(IrohError::doc)?;

            Ok(content.to_vec())
        })
    }

    /// Set an entries on the doc via its key, hash, and size.
    pub fn set_hash(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        hash: Arc<Hash>,
        size: u64,
    ) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner
                .set_hash(author_id.0, key, hash.0, size)
                .await
                .map_err(IrohError::doc)?;
            Ok(())
        })
    }

    /// Get the content size of an [`Entry`]
    pub fn size(&self, entry: Arc<Entry>) -> Result<u64, IrohError> {
        block_on(&self.rt, async {
            let r = self.inner.read(&entry.0).await.map_err(IrohError::doc)?;
            Ok(r.size())
        })
    }

    /// Read all content of an [`Entry`] into a buffer.
    /// This allocates a buffer for the full entry. Use only if you know that the entry you're
    /// reading is small. If not sure, use [`Self::size`] and check the size with
    /// before calling [`Self::read_to_bytes`].
    pub fn read_to_bytes(&self, entry: Arc<Entry>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .read_to_bytes(&entry.0)
                .await
                .map(|c| c.to_vec())
                .map_err(IrohError::doc)
        })
    }

    /// Delete entries that match the given `author` and key `prefix`.
    ///
    /// This inserts an empty entry with the key set to `prefix`, effectively clearing all other
    /// entries whose key starts with or is equal to the given `prefix`.
    ///
    /// Returns the number of entries deleted.
    pub fn del(&self, author_id: Arc<AuthorId>, prefix: Vec<u8>) -> Result<u64, IrohError> {
        block_on(&self.rt, async {
            let num_del = self
                .inner
                .del(author_id.0, prefix)
                .await
                .map_err(IrohError::doc)?;
            u64::try_from(num_del).map_err(IrohError::doc)
        })
    }

    /// Get the latest entry for a key and author.
    pub fn get_one(&self, query: Arc<Query>) -> Result<Option<Arc<Entry>>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .get_one((*query).clone().0)
                .await
                .map(|e| e.map(|e| Arc::new(e.into())))
                .map_err(IrohError::doc)
        })
    }

    /// Get entries.
    ///
    /// Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn get_many(&self, query: Arc<Query>) -> Result<Vec<Arc<Entry>>, IrohError> {
        block_on(&self.rt, async {
            let entries = self
                .inner
                .get_many(query.0.clone())
                .await
                .map_err(IrohError::doc)?
                .map_ok(|e| Arc::new(Entry(e)))
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::doc)?;
            Ok(entries)
        })
    }

    /// Share this document with peers over a ticket.
    pub fn share(&self, mode: ShareMode) -> anyhow::Result<Arc<DocTicket>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .share(mode.into())
                .await
                .map(|ticket| Arc::new(DocTicket(ticket)))
                .map_err(IrohError::doc)
        })
    }

    /// Start to sync this document with a list of peers.
    pub fn start_sync(&self, peers: Vec<Arc<NodeAddr>>) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner
                .start_sync(peers.into_iter().map(|p| (*p).clone().into()).collect())
                .await
                .map_err(IrohError::doc)
        })
    }

    /// Stop the live sync for this document.
    pub fn leave(&self) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner.leave().await.map_err(IrohError::doc)
        })
    }

    /// Get status info for this document
    pub fn status(&self) -> Result<OpenState, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .status()
                .await
                .map(|o| o.into())
                .map_err(IrohError::doc)
        })
    }

    pub fn subscribe(&self, cb: Box<dyn SubscribeCallback>) -> Result<(), IrohError> {
        let client = self.inner.clone();
        self.rt.main().spawn(async move {
            let mut sub = client.subscribe().await.unwrap();
            while let Some(event) = sub.next().await {
                match event {
                    Ok(event) => {
                        if let Err(err) = cb.event(Arc::new(event.into())) {
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

/// The state for an open replica.
#[derive(Debug, Clone, Copy)]
pub struct OpenState {
    /// Whether to accept sync requests for this replica.
    pub sync: bool,
    /// How many event subscriptions are open
    pub subscribers: u64,
    /// By how many handles the replica is currently held open
    pub handles: u64,
}

impl From<iroh::sync::actor::OpenState> for OpenState {
    fn from(value: iroh::sync::actor::OpenState) -> Self {
        OpenState {
            sync: value.sync,
            subscribers: value.subscribers as u64,
            handles: value.handles as u64,
        }
    }
}

/// A peer and it's addressing information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAddr {
    node_id: Arc<PublicKey>,
    derp_region: Option<u16>,
    addresses: Vec<Arc<SocketAddr>>,
}

impl NodeAddr {
    /// Create a new [`NodeAddr`] with empty [`AddrInfo`].
    pub fn new(
        node_id: Arc<PublicKey>,
        derp_region: Option<u16>,
        addresses: Vec<Arc<SocketAddr>>,
    ) -> Self {
        Self {
            node_id,
            derp_region,
            addresses,
        }
    }

    /// Get the direct addresses of this peer.
    pub fn direct_addresses(&self) -> Vec<Arc<SocketAddr>> {
        self.addresses.clone()
    }

    /// Get the derp region of this peer.
    pub fn derp_region(&self) -> Option<u16> {
        self.derp_region
    }

    /// Returns true if both NodeAddr's have the same values
    pub fn equal(&self, other: Arc<NodeAddr>) -> bool {
        *self == *other
    }
}

impl From<NodeAddr> for iroh::net::magic_endpoint::NodeAddr {
    fn from(value: NodeAddr) -> Self {
        let mut node_addr = iroh::net::magic_endpoint::NodeAddr::new(value.node_id.0);
        let addresses = value.direct_addresses().into_iter().map(|addr| {
            let typ = addr.r#type();
            match typ {
                SocketAddrType::V4 => {
                    let addr_str = addr.to_string();
                    std::net::SocketAddrV4::from_str(&addr_str)
                        .expect("checked")
                        .into()
                }
                SocketAddrType::V6 => {
                    let addr_str = addr.to_string();
                    std::net::SocketAddrV6::from_str(&addr_str)
                        .expect("checked")
                        .into()
                }
            }
        });
        if let Some(derp_region) = value.derp_region() {
            node_addr = node_addr.with_derp_region(derp_region);
        }
        node_addr = node_addr.with_direct_addresses(addresses);
        node_addr
    }
}

/// Intended capability for document share tickets
#[derive(Debug)]
pub enum ShareMode {
    /// Read-only access
    Read,
    /// Write access
    Write,
}

impl From<ShareMode> for iroh::rpc_protocol::ShareMode {
    fn from(mode: ShareMode) -> Self {
        match mode {
            ShareMode::Read => iroh::rpc_protocol::ShareMode::Read,
            ShareMode::Write => iroh::rpc_protocol::ShareMode::Write,
        }
    }
}

/// A single entry in a [`Doc`]
///
/// An entry is identified by a key, its [`AuthorId`], and the [`Doc`]'s
/// [`NamespaceId`]. Its value is the 32-byte BLAKE3 [`hash`]
/// of the entry's content data, the size of this content data, and a timestamp.
#[derive(Debug, Clone)]
pub struct Entry(pub(crate) iroh::sync::Entry);

impl From<iroh::sync::Entry> for Entry {
    fn from(e: iroh::sync::Entry) -> Self {
        Entry(e)
    }
}

impl Entry {
    /// Get the [`AuthorId`] of this entry.
    pub fn author(&self) -> Arc<AuthorId> {
        Arc::new(AuthorId(self.0.id().author()))
    }

    /// Get the key of this entry.
    pub fn key(&self) -> Vec<u8> {
        self.0.id().key().to_vec()
    }

    /// Get the [`NamespaceId`] of this entry.
    pub fn namespace(&self) -> Arc<NamespaceId> {
        Arc::new(NamespaceId(self.0.id().namespace()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorId(pub(crate) iroh::sync::AuthorId);

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AuthorId {
    /// Get an [`AuthorId`] from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::sync::AuthorId::from_str(&str).map_err(IrohError::author)?;
        Ok(AuthorId(author))
    }

    /// Returns true when both AuthorId's have the same value
    pub fn equal(&self, other: Arc<AuthorId>) -> bool {
        *self == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceId(pub(crate) iroh::sync::NamespaceId);

impl From<iroh::sync::NamespaceId> for NamespaceId {
    fn from(id: iroh::sync::NamespaceId) -> Self {
        NamespaceId(id)
    }
}

impl NamespaceId {
    /// Get an [`NamespaceId`] from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::sync::NamespaceId::from_str(&str).map_err(IrohError::namespace)?;
        Ok(NamespaceId(author))
    }

    /// Returns true when both NamespaceId's have the same value
    pub fn equal(&self, other: Arc<NamespaceId>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub use iroh::sync::store::SortBy;
pub use iroh::sync::store::SortDirection;

#[derive(Clone, Debug)]
pub struct Query(iroh::sync::store::Query);

impl Query {
    /// Query all records.
    pub fn all(
        sort_by: SortBy,
        direction: SortDirection,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        let mut builder = iroh::sync::store::Query::all().sort_by(sort_by, direction);
        if let Some(offset) = offset {
            builder = builder.offset(offset);
        }
        if let Some(limit) = limit {
            builder = builder.limit(limit);
        }
        Query(builder.build())
    }

    /// Query only the latest entry for each key, omitting older entries if the entry was written
    /// to by multiple authors.
    pub fn single_latest_per_key(
        direction: SortDirection,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        let mut builder =
            iroh::sync::store::Query::single_latest_per_key().sort_direction(direction);
        if let Some(offset) = offset {
            builder = builder.offset(offset);
        }
        if let Some(limit) = limit {
            builder = builder.limit(limit);
        }
        Query(builder.build())
    }

    /// Create a [`Query::all`] query filtered by a single author.
    pub fn author(
        author: Arc<AuthorId>,
        sort_by: SortBy,
        direction: SortDirection,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        let mut builder =
            iroh::sync::store::Query::author((*author).0.clone()).sort_by(sort_by, direction);
        if let Some(offset) = offset {
            builder = builder.offset(offset);
        }
        if let Some(limit) = limit {
            builder = builder.limit(limit);
        }
        Query(builder.build())
    }

    /// Create a [`Query::all`] query filtered by a single key.
    pub fn key_exact(
        key: Vec<u8>,
        sort_by: SortBy,
        direction: SortDirection,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        let mut builder = iroh::sync::store::Query::key_exact(&key).sort_by(sort_by, direction);
        if let Some(offset) = offset {
            builder = builder.offset(offset);
        }
        if let Some(limit) = limit {
            builder = builder.limit(limit);
        }
        Query(builder.build())
    }

    /// Create a [`Query::all`] query filtered by a key prefix.
    pub fn key_prefix(
        prefix: Vec<u8>,
        sort_by: SortBy,
        direction: SortDirection,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        let mut builder = iroh::sync::store::Query::key_prefix(&prefix).sort_by(sort_by, direction);
        if let Some(offset) = offset {
            builder = builder.offset(offset);
        }
        if let Some(limit) = limit {
            builder = builder.limit(limit);
        }
        Query(builder.build())
    }

    /// Get the limit for this query (max. number of entries to emit).
    pub fn limit(&self) -> Option<u64> {
        self.0.limit()
    }

    /// Get the offset for this query (number of entries to skip at the beginning).
    pub fn offset(&self) -> u64 {
        self.0.offset()
    }
}

/// Contains both a key (either secret or public) to a document, and a list of peers to join.
#[derive(Debug, Clone)]
pub struct DocTicket(pub(crate) iroh::rpc_protocol::DocTicket);

impl DocTicket {
    /// Create a `DocTicket` from a string
    pub fn from_string(content: String) -> Result<Self, IrohError> {
        let ticket = content
            .parse::<iroh::rpc_protocol::DocTicket>()
            .map_err(IrohError::doc_ticket)?;
        Ok(DocTicket(ticket))
    }

    /// Returns true if both `DocTicket`'s have the same value
    pub fn equal(&self, other: Arc<DocTicket>) -> bool {
        // TODO: implement partialeq and eq on DocTicket
        self.to_string() == *other.to_string()
    }
}

impl std::fmt::Display for DocTicket {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait SubscribeCallback: Send + Sync + 'static {
    fn event(&self, event: Arc<LiveEvent>) -> Result<(), IrohError>;
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

impl From<iroh::sync::ContentStatus> for ContentStatus {
    fn from(value: iroh::sync::ContentStatus) -> Self {
        match value {
            iroh::sync::ContentStatus::Complete => Self::Complete,
            iroh::sync::ContentStatus::Incomplete => Self::Incomplete,
            iroh::sync::ContentStatus::Missing => Self::Missing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ipv4Addr, Ipv6Addr, PublicKey, SocketAddr};

    #[test]
    fn test_node_addr() {
        //
        // create a node_id
        let key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva";
        let node_id = PublicKey::from_string(key_str.into()).unwrap();
        //
        // create socketaddrs
        let ipv4_ip = Ipv4Addr::from_string("127.0.0.1".into()).unwrap();
        let ipv6_ip = Ipv6Addr::from_string("::1".into()).unwrap();
        let port = 3000;
        //
        // create socket addrs
        let ipv4 = SocketAddr::from_ipv4(ipv4_ip.into(), port);
        let ipv6 = SocketAddr::from_ipv6(ipv6_ip.into(), port);
        //
        // derp region
        let derp_region = Some(1);
        //
        // create a NodeAddr
        let addrs = vec![Arc::new(ipv4), Arc::new(ipv6)];
        let expect_addrs = addrs.clone();
        let node_addr = NodeAddr::new(node_id.into(), derp_region, addrs);
        //
        // test we have returned the expected addresses
        let got_addrs = node_addr.direct_addresses();
        let addrs = expect_addrs.iter().zip(got_addrs.iter());
        for (expect, got) in addrs {
            assert!(got.equal(expect.clone()));
            assert!(expect.equal(got.clone()));
        }

        assert_eq!(derp_region, node_addr.derp_region());
    }
    #[test]
    fn test_namespace_id() {
        //
        // create id from string
        let namespace_str = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q";
        let namespace = NamespaceId::from_string(namespace_str.into()).unwrap();
        //
        // call to_string, ensure equal
        assert_eq!(namespace.to_string(), namespace_str);
        //
        // create another id, same string
        let namespace_0 = NamespaceId::from_string(namespace_str.into()).unwrap();
        //
        // ensure equal
        let namespace_0 = Arc::new(namespace_0);
        assert!(namespace.equal(namespace_0.clone()));
        assert!(namespace_0.equal(namespace.into()));
    }
    #[test]
    fn test_author_id() {
        //
        // create id from string
        let author_str = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q";
        let author = AuthorId::from_string(author_str.into()).unwrap();
        //
        // call to_string, ensure equal
        assert_eq!(author_str, author.to_string());
        //
        // create another id, same string
        let author_0 = AuthorId::from_string(author_str.into()).unwrap();
        //
        // ensure equal
        let author_0 = Arc::new(author_0);
        assert!(author.equal(author_0.clone()));
        assert!(author_0.equal(author.into()));
    }
    #[test]
    fn test_doc_ticket() {
        //
        // create id from string
        let doc_ticket_str = "docaaqjjfgbzx2ry4zpaoujdppvqktgvfvpxgqubkghiialqovv7z4wosqbebpvjjp2tywajvg6unjza6dnugkalg4srmwkcucmhka7mgy4r3aa4aibayaeusjsjlcfoagavaa4xrcxaetag4aaq45mxvqaaaaaaaaadiu4kvybeybxaaehhlf5mdenfufmhk7nixcvoajganyabbz2zplgbno2vsnuvtkpyvlqcjqdoaaioowl22k3fc26qjx4ot6fk4";
        let doc_ticket = DocTicket::from_string(doc_ticket_str.into()).unwrap();
        //
        // call to_string, ensure equal
        assert_eq!(doc_ticket_str, doc_ticket.to_string());
        //
        // create another id, same string
        let doc_ticket_0 = DocTicket::from_string(doc_ticket_str.into()).unwrap();
        //
        // ensure equal
        let doc_ticket_0 = Arc::new(doc_ticket_0);
        assert!(doc_ticket.equal(doc_ticket_0.clone()));
        assert!(doc_ticket_0.equal(doc_ticket.into()));
    }

    #[test]
    fn test_query() {
        // all
        let all = Query::all(SortBy::KeyAuthor, SortDirection::Asc, Some(10), Some(10));
        assert_eq!(10, all.offset());
        assert_eq!(Some(10), all.limit());

        let single_latest_per_key = Query::single_latest_per_key(SortDirection::Desc, None, None);
        assert_eq!(0, single_latest_per_key.offset());
        assert_eq!(None, single_latest_per_key.limit());

        let author = Query::author(
            Arc::new(
                AuthorId::from_string(
                    "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q".to_string(),
                )
                .unwrap(),
            ),
            SortBy::AuthorKey,
            SortDirection::Asc,
            Some(100),
            None,
        );
        assert_eq!(100, author.offset());
        assert_eq!(None, author.limit());

        let key_exact = Query::key_exact(
            b"key".to_vec(),
            SortBy::KeyAuthor,
            SortDirection::Desc,
            None,
            Some(100),
        );
        assert_eq!(0, key_exact.offset());
        assert_eq!(Some(100), key_exact.limit());

        let key_prefix = Query::key_prefix(
            b"prefix".to_vec(),
            SortBy::KeyAuthor,
            SortDirection::Desc,
            None,
            Some(100),
        );
        assert_eq!(0, key_prefix.offset());
        assert_eq!(Some(100), key_prefix.limit());
    }
}
