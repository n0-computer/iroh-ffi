use std::{str::FromStr, sync::Arc, time::SystemTime};

use futures::{StreamExt, TryStreamExt};
use iroh::{
    client::Doc as ClientDoc,
    rpc_protocol::{ProviderRequest, ProviderResponse},
};

pub use iroh::sync::CapabilityKind;

use quic_rpc::transport::flume::FlumeConnection;

use crate::runtime::Handle;
use crate::{
    block_on, AuthorId, Hash, IrohError, IrohNode, PublicKey, SocketAddr, SocketAddrType, Url,
};

impl IrohNode {
    /// Create a new doc.
    pub fn doc_create(&self) -> Result<Arc<Doc>, IrohError> {
        block_on(&self.async_runtime, async {
            let doc = self
                .sync_client
                .docs
                .create()
                .await
                .map_err(IrohError::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.async_runtime.clone(),
            }))
        })
    }

    /// Join and sync with an already existing document.
    pub fn doc_join(&self, ticket: Arc<DocTicket>) -> Result<Arc<Doc>, IrohError> {
        block_on(&self.async_runtime, async {
            let doc = self
                .sync_client
                .docs
                .import(ticket.0.clone())
                .await
                .map_err(IrohError::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.async_runtime.clone(),
            }))
        })
    }

    /// List all the docs we have access to on this node.
    pub fn doc_list(&self) -> Result<Vec<NamespaceAndCapability>, IrohError> {
        block_on(&self.async_runtime, async {
            let docs = self
                .sync_client
                .docs
                .list()
                .await
                .map_err(IrohError::doc)?
                .map_ok(|(namespace, capability)| NamespaceAndCapability {
                    namespace: Arc::new(namespace.into()),
                    capability,
                })
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::doc)?;

            Ok(docs)
        })
    }

    /// Get a [`Doc`].
    ///
    /// Returns None if the document cannot be found.
    pub fn doc_open(&self, id: Arc<NamespaceId>) -> Result<Option<Arc<Doc>>, IrohError> {
        block_on(&self.async_runtime, async {
            let doc = self
                .sync_client
                .docs
                .open(id.0)
                .await
                .map_err(IrohError::doc)?;
            Ok(doc.map(|d| {
                Arc::new(Doc {
                    inner: d,
                    rt: self.async_runtime.clone(),
                })
            }))
        })
    }

    /// Delete a document from the local node.
    ///
    /// This is a destructive operation. Both the document secret key and all entries in the
    /// document will be permanently deleted from the node's storage. Content blobs will be
    /// deleted.clone()).await.map_err(Iroh::doc)
    /// through garbage collection unless they are referenced from another document or tag.
    pub fn doc_drop(&self, doc_id: Arc<NamespaceId>) -> Result<(), IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .docs
                .drop_doc(doc_id.0)
                .await
                .map_err(IrohError::doc)
        })
    }
}

/// The NamespaceId and CapabilityKind (read/write) of the doc
pub struct NamespaceAndCapability {
    /// The NamespaceId of the doc
    pub namespace: Arc<NamespaceId>,
    /// The capability you have for the doc (read/write)
    pub capability: CapabilityKind,
}

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

    /// Add an entry from an absolute file path
    pub fn import_file(
        &self,
        author: Arc<AuthorId>,
        key: Vec<u8>,
        path: String,
        in_place: bool,
        cb: Option<Box<dyn DocImportFileCallback>>,
    ) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            let mut stream = self
                .inner
                .import_file(
                    author.0,
                    bytes::Bytes::from(key),
                    std::path::PathBuf::from(path),
                    in_place,
                )
                .await
                .map_err(IrohError::doc)?;
            while let Some(progress) = stream.next().await {
                let progress = progress.map_err(IrohError::doc)?;
                if let Some(ref cb) = cb {
                    cb.progress(Arc::new(progress.into()))?;
                }
            }
            Ok(())
        })
    }

    /// Export an entry as a file to a given absolute path
    pub fn export_file(
        &self,
        entry: Arc<Entry>,
        path: String,
        cb: Option<Box<dyn DocExportFileCallback>>,
    ) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            let mut stream = self
                .inner
                .export_file(entry.0.clone(), std::path::PathBuf::from(path))
                .await
                .map_err(IrohError::doc)?;
            while let Some(progress) = stream.next().await {
                let progress = progress.map_err(IrohError::doc)?;
                if let Some(ref cb) = cb {
                    cb.progress(Arc::new(progress.into()))?
                }
            }
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

    /// Get an entry for a key and author.
    pub fn get_exact(
        &self,
        author: Arc<AuthorId>,
        key: Vec<u8>,
        include_empty: bool,
    ) -> Result<Option<Arc<Entry>>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .get_exact(author.0, key, include_empty)
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
    pub fn share(&self, mode: ShareMode) -> Result<Arc<DocTicket>, IrohError> {
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

    /// Subscribe to events for this document.
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
    derp_url: Option<Arc<Url>>,
    addresses: Vec<Arc<SocketAddr>>,
}

impl NodeAddr {
    /// Create a new [`NodeAddr`] with empty [`AddrInfo`].
    pub fn new(
        node_id: Arc<PublicKey>,
        derp_url: Option<Arc<Url>>,
        addresses: Vec<Arc<SocketAddr>>,
    ) -> Self {
        Self {
            node_id,
            derp_url,
            addresses,
        }
    }

    /// Get the direct addresses of this peer.
    pub fn direct_addresses(&self) -> Vec<Arc<SocketAddr>> {
        self.addresses.clone()
    }

    /// Get the derp region of this peer.
    pub fn derp_url(&self) -> Option<Arc<Url>> {
        self.derp_url
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
        if let Some(derp_url) = value.derp_url() {
            node_addr = node_addr.with_derp_url(derp_url.0.clone());
        }
        node_addr = node_addr.with_direct_addresses(addresses);
        node_addr
    }
}

impl From<iroh::net::magic_endpoint::NodeAddr> for NodeAddr {
    fn from(value: iroh::net::magic_endpoint::NodeAddr) -> Self {
        NodeAddr {
            node_id: Arc::new(value.node_id.into()),
            derp_url: value.info.derp_url.map(|url| Arc::new(url.into())),
            addresses: value
                .info
                .direct_addresses
                .into_iter()
                .map(|d| Arc::new(d.into()))
                .collect(),
        }
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

    /// Get the content_hash of this entry.
    pub fn content_hash(&self) -> Arc<Hash> {
        Arc::new(Hash(self.0.content_hash()))
    }

    /// Get the content_length of this entry.
    pub fn content_len(&self) -> u64 {
        self.0.content_len()
    }
}

/// An identifier for a Doc
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

/// Fields by which the query can be sorted
pub use iroh::sync::store::SortBy;

/// Sort direction
pub use iroh::sync::store::SortDirection;

/// Build a Query to search for an entry or entries in a doc.
///
/// Use this with `QueryOptions` to determine sorting, grouping, and pagination.
#[derive(Clone, Debug)]
pub struct Query(iroh::sync::store::Query);

/// Options for sorting and pagination for using [`Query`]s.
#[derive(Clone, Debug, Default)]
pub struct QueryOptions {
    /// Sort by author or key first.
    ///
    /// Default is [`SortBy::AuthorKey`], so sorting first by author and then by key.
    pub sort_by: SortBy,
    /// Direction by which to sort the entries
    ///
    /// Default is [`SortDirection::Asc`]
    pub direction: SortDirection,
    /// Offset
    pub offset: u64,
    /// Limit to limit the pagination.
    ///
    /// When the limit is 0, the limit does not exist.
    pub limit: u64,
}

impl Query {
    /// Query all records.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn all(opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::all();

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by, opts.direction);
        }
        Query(builder.build())
    }

    /// Query only the latest entry for each key, omitting older entries if the entry was written
    /// to by multiple authors.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn single_latest_per_key(opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::single_latest_per_key();

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_direction(opts.direction);
        }
        Query(builder.build())
    }

    /// Query all entries for by a single author.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn author(author: Arc<AuthorId>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::author(author.0);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by, opts.direction);
        }
        Query(builder.build())
    }

    /// Query all entries that have an exact key.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn key_exact(key: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::key_exact(key);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by, opts.direction);
        }
        Query(builder.build())
    }

    /// Create a Query for a single key and author.
    pub fn author_key_exact(author: Arc<AuthorId>, key: Vec<u8>) -> Self {
        let builder = iroh::sync::store::Query::author(author.0).key_exact(key);
        Query(builder.build())
    }

    /// Create a query for all entries with a given key prefix.
    ///  
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn key_prefix(prefix: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::key_prefix(prefix);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by, opts.direction);
        }
        Query(builder.build())
    }

    /// Create a query for all entries of a single author with a given key prefix.
    ///  
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn author_key_prefix(
        author: Arc<AuthorId>,
        prefix: Vec<u8>,
        opts: Option<QueryOptions>,
    ) -> Self {
        let mut builder = iroh::sync::store::Query::author(author.0).key_prefix(prefix);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by, opts.direction);
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

/// The `progress` method will be called for each `SubscribeProgress` event that is
/// emitted during a `node.doc_subscribe`. Use the `SubscribeProgress.type()`
/// method to check the `LiveEvent`
pub trait SubscribeCallback: Send + Sync + 'static {
    fn event(&self, event: Arc<LiveEvent>) -> Result<(), IrohError>;
}

/// Events informing about actions of the live sync progress
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

/// The type of events that can be emitted during the live sync progress
pub enum LiveEventType {
    /// A local insertion.
    InsertLocal,
    /// Received a remote insert.
    InsertRemote,
    /// The content of an entry was downloaded and is now available at the local node
    ContentReady,
    /// We have a new neighbor in the swarm.
    NeighborUp,
    /// We lost a neighbor in the swarm.
    NeighborDown,
    /// A set-reconciliation sync finished.
    SyncFinished,
}

impl LiveEvent {
    /// The type LiveEvent
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

    /// For `LiveEventType::InsertLocal`, returns an Entry
    pub fn as_insert_local(&self) -> Arc<Entry> {
        if let Self::InsertLocal { entry } = self {
            Arc::new(entry.clone())
        } else {
            panic!("not an insert local event");
        }
    }

    /// For `LiveEventType::InsertRemote`, returns an InsertRemoteEvent
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

    /// For `LiveEventType::ContentReady`, returns a Hash
    pub fn as_content_ready(&self) -> Arc<Hash> {
        if let Self::ContentReady { hash } = self {
            Arc::new(hash.clone())
        } else {
            panic!("not an content ready event");
        }
    }

    /// For `LiveEventType::NeighborUp`, returns a PublicKey
    pub fn as_neighbor_up(&self) -> Arc<PublicKey> {
        if let Self::NeighborUp(key) = self {
            Arc::new(key.clone())
        } else {
            panic!("not an neighbor up event");
        }
    }

    /// For `LiveEventType::NeighborDown`, returns a PublicKey
    pub fn as_neighbor_down(&self) -> Arc<PublicKey> {
        if let Self::NeighborDown(key) = self {
            Arc::new(key.clone())
        } else {
            panic!("not an neighbor down event");
        }
    }

    /// For `LiveEventType::SyncFinished`, returns a SyncEvent
    pub fn as_sync_finished(&self) -> SyncEvent {
        if let Self::SyncFinished(event) = self {
            event.clone()
        } else {
            panic!("not an sync event event");
        }
    }
}

impl From<iroh::client::LiveEvent> for LiveEvent {
    fn from(value: iroh::client::LiveEvent) -> Self {
        match value {
            iroh::client::LiveEvent::InsertLocal { entry } => LiveEvent::InsertLocal {
                entry: entry.into(),
            },
            iroh::client::LiveEvent::InsertRemote {
                from,
                entry,
                content_status,
            } => LiveEvent::InsertRemote {
                from: from.into(),
                entry: entry.into(),
                content_status: content_status.into(),
            },
            iroh::client::LiveEvent::ContentReady { hash } => {
                LiveEvent::ContentReady { hash: hash.into() }
            }
            iroh::client::LiveEvent::NeighborUp(key) => LiveEvent::NeighborUp(key.into()),
            iroh::client::LiveEvent::NeighborDown(key) => LiveEvent::NeighborDown(key.into()),
            iroh::client::LiveEvent::SyncFinished(e) => LiveEvent::SyncFinished(e.into()),
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

/// Why we started a sync request
pub use iroh::sync_engine::SyncReason;

/// Why we performed a sync exchange
#[derive(Debug, Clone)]
pub enum Origin {
    /// public, use a unit variant
    Connect { reason: SyncReason },
    /// A peer connected to us and we accepted the exchange
    Accept,
}

impl From<iroh::sync_engine::Origin> for Origin {
    fn from(value: iroh::sync_engine::Origin) -> Self {
        match value {
            iroh::sync_engine::Origin::Connect(reason) => Self::Connect { reason },
            iroh::sync_engine::Origin::Accept => Self::Accept,
        }
    }
}

/// Outcome of an InsertRemove event.
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

/// The `progress` method will be called for each `DocImportProgress` event that is
/// emitted during a `doc.import_file()` call. Use the `DocImportProgress.type()`
/// method to check the `DocImportProgressType`
pub trait DocImportFileCallback: Send + Sync + 'static {
    fn progress(&self, progress: Arc<DocImportProgress>) -> Result<(), IrohError>;
}

/// The type of `DocImportProgress` event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocImportProgressType {
    /// An item was found with name `name`, from now on referred to via `id`
    Found,
    /// We got progress ingesting item `id`.
    Progress,
    /// We are done ingesting `id`, and the hash is `hash`.
    IngestDone,
    /// We are done with the whole operation.
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort,
}

/// A DocImportProgress event indicating a file was found with name `name`, from now on referred to via `id`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocImportProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// A DocImportProgress event indicating we've made progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocImportProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DocImportProgress event indicating we are finished adding `id` to the data store and the hash is `hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocImportProgressIngestDone {
    /// The unique id of the entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// A DocImportProgress event indicating we are done setting the entry to the doc
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocImportProgressAllDone {
    /// The key of the entry
    pub key: Vec<u8>,
}

/// A DocImportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocImportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocImportProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    Found(DocImportProgressFound),
    /// We got progress ingesting item `id`.
    Progress(DocImportProgressProgress),
    /// We are done ingesting `id`, and the hash is `hash`.
    IngestDone(DocImportProgressIngestDone),
    /// We are done with the whole operation.
    AllDone(DocImportProgressAllDone),
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort(DocImportProgressAbort),
}

impl From<iroh::rpc_protocol::DocImportProgress> for DocImportProgress {
    fn from(value: iroh::rpc_protocol::DocImportProgress) -> Self {
        match value {
            iroh::rpc_protocol::DocImportProgress::Found { id, name, size } => {
                DocImportProgress::Found(DocImportProgressFound { id, name, size })
            }
            iroh::rpc_protocol::DocImportProgress::Progress { id, offset } => {
                DocImportProgress::Progress(DocImportProgressProgress { id, offset })
            }
            iroh::rpc_protocol::DocImportProgress::IngestDone { id, hash } => {
                DocImportProgress::IngestDone(DocImportProgressIngestDone {
                    id,
                    hash: Arc::new(hash.into()),
                })
            }
            iroh::rpc_protocol::DocImportProgress::AllDone { key } => {
                DocImportProgress::AllDone(DocImportProgressAllDone { key: key.into() })
            }
            iroh::rpc_protocol::DocImportProgress::Abort(err) => {
                DocImportProgress::Abort(DocImportProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

impl DocImportProgress {
    /// Get the type of event
    pub fn r#type(&self) -> DocImportProgressType {
        match self {
            DocImportProgress::Found(_) => DocImportProgressType::Found,
            DocImportProgress::Progress(_) => DocImportProgressType::Progress,
            DocImportProgress::IngestDone(_) => DocImportProgressType::IngestDone,
            DocImportProgress::AllDone(_) => DocImportProgressType::AllDone,
            DocImportProgress::Abort(_) => DocImportProgressType::Abort,
        }
    }

    /// Return the `DocImportProgressFound` event
    pub fn as_found(&self) -> DocImportProgressFound {
        match self {
            DocImportProgress::Found(f) => f.clone(),
            _ => panic!("DocImportProgress type is not 'Found'"),
        }
    }

    /// Return the `DocImportProgressProgress` event
    pub fn as_progress(&self) -> DocImportProgressProgress {
        match self {
            DocImportProgress::Progress(p) => p.clone(),
            _ => panic!("DocImportProgress type is not 'Progress'"),
        }
    }

    /// Return the `DocImportProgressDone` event
    pub fn as_ingest_done(&self) -> DocImportProgressIngestDone {
        match self {
            DocImportProgress::IngestDone(d) => d.clone(),
            _ => panic!("DocImportProgress type is not 'IngestDone'"),
        }
    }

    /// Return the `DocImportProgressAllDone`
    pub fn as_all_done(&self) -> DocImportProgressAllDone {
        match self {
            DocImportProgress::AllDone(a) => a.clone(),
            _ => panic!("DocImportProgress type is not 'AllDone'"),
        }
    }

    /// Return the `DocImportProgressAbort`
    pub fn as_abort(&self) -> DocImportProgressAbort {
        match self {
            DocImportProgress::Abort(a) => a.clone(),
            _ => panic!("DocImportProgress type is not 'Abort'"),
        }
    }
}

/// The `progress` method will be called for each `DocExportProgress` event that is
/// emitted during a `doc.export_file()` call. Use the `DocExportProgress.type()`
/// method to check the `DocExportProgressType`
pub trait DocExportFileCallback: Send + Sync + 'static {
    fn progress(&self, progress: Arc<DocExportProgress>) -> Result<(), IrohError>;
}

/// The type of `DocExportProgress` event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocExportProgressType {
    /// An item was found with name `name`, from now on referred to via `id`
    Found,
    /// We got progress exporting item `id`.
    Progress,
    /// We are done writing the entry to the filesystem
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort,
}

/// A DocExportProgress event indicating a file was found with name `name`, from now on referred to via `id`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocExportProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
    /// The key of the entry.
    pub key: Vec<u8>,
    /// The size of the entry in bytes.
    pub size: u64,
    /// The path where we are writing the entry
    pub outpath: String,
}

/// A DocExportProgress event indicating we've made progress exporting item `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocExportProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DocExportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocExportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocExportProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    Found(DocExportProgressFound),
    /// We got progress ingesting item `id`.
    Progress(DocExportProgressProgress),
    /// We are done with the whole operation.
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort(DocExportProgressAbort),
}

impl From<iroh::rpc_protocol::DocExportProgress> for DocExportProgress {
    fn from(value: iroh::rpc_protocol::DocExportProgress) -> Self {
        match value {
            iroh::rpc_protocol::DocExportProgress::Found {
                id,
                hash,
                key,
                size,
                outpath,
            } => DocExportProgress::Found(DocExportProgressFound {
                id,
                hash: Arc::new(hash.into()),
                key: key.to_vec(),
                size,
                outpath: outpath.to_string_lossy().to_string(),
            }),
            iroh::rpc_protocol::DocExportProgress::Progress { id, offset } => {
                DocExportProgress::Progress(DocExportProgressProgress { id, offset })
            }
            iroh::rpc_protocol::DocExportProgress::AllDone => DocExportProgress::AllDone,
            iroh::rpc_protocol::DocExportProgress::Abort(err) => {
                DocExportProgress::Abort(DocExportProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

impl DocExportProgress {
    /// Get the type of event
    pub fn r#type(&self) -> DocExportProgressType {
        match self {
            DocExportProgress::Found(_) => DocExportProgressType::Found,
            DocExportProgress::Progress(_) => DocExportProgressType::Progress,
            DocExportProgress::AllDone => DocExportProgressType::AllDone,
            DocExportProgress::Abort(_) => DocExportProgressType::Abort,
        }
    }
    /// Return the `DocExportProgressFound` event
    pub fn as_found(&self) -> DocExportProgressFound {
        match self {
            DocExportProgress::Found(f) => f.clone(),
            _ => panic!("DocExportProgress type is not 'Found'"),
        }
    }
    /// Return the `DocExportProgressProgress` event
    pub fn as_progress(&self) -> DocExportProgressProgress {
        match self {
            DocExportProgress::Progress(p) => p.clone(),
            _ => panic!("DocExportProgress type is not 'Progress'"),
        }
    }
    /// Return the `DocExportProgressAbort`
    pub fn as_abort(&self) -> DocExportProgressAbort {
        match self {
            DocExportProgress::Abort(a) => a.clone(),
            _ => panic!("DocExportProgress type is not 'Abort'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ipv4Addr, Ipv6Addr, PublicKey, SocketAddr};
    use rand::RngCore;
    use std::io::Write;

    #[test]
    fn test_doc_create() {
        let path = tempfile::tempdir().unwrap();
        let node = IrohNode::new(path.path().to_string_lossy().into_owned()).unwrap();
        let node_id = node.node_id();
        println!("id: {}", node_id);
        let doc = node.doc_create().unwrap();
        let doc_id = doc.id();
        println!("doc_id: {}", doc_id);

        let doc_ticket = doc.share(crate::doc::ShareMode::Write).unwrap();
        let doc_ticket_string = doc_ticket.to_string();
        let dock_ticket_back = DocTicket::from_string(doc_ticket_string.clone()).unwrap();
        assert_eq!(doc_ticket.0.to_string(), dock_ticket_back.0.to_string());
        println!("doc_ticket: {}", doc_ticket_string);
        node.doc_join(doc_ticket).unwrap();
    }

    #[test]
    fn test_basic_sync() {
        // create node_0
        let iroh_dir_0 = tempfile::tempdir().unwrap();
        let node_0 = IrohNode::new(iroh_dir_0.path().to_string_lossy().into_owned()).unwrap();

        // create node_1
        let iroh_dir_1 = tempfile::tempdir().unwrap();
        let node_1 = IrohNode::new(iroh_dir_1.path().to_string_lossy().into_owned()).unwrap();

        // create doc on node_0
        let doc_0 = node_0.doc_create().unwrap();
        let ticket = doc_0.share(ShareMode::Write).unwrap();

        // subscribe to sync events
        let (found_s, found_r) = std::sync::mpsc::channel();
        struct Callback {
            found_s: std::sync::mpsc::Sender<Result<Hash, IrohError>>,
        }
        impl SubscribeCallback for Callback {
            fn event(&self, event: Arc<LiveEvent>) -> Result<(), IrohError> {
                match *event {
                    LiveEvent::ContentReady { ref hash } => {
                        self.found_s
                            .send(Ok(hash.clone()))
                            .map_err(IrohError::doc)?;
                    }
                    _ => {}
                }
                Ok(())
            }
        }
        let cb = Callback { found_s };
        doc_0.subscribe(Box::new(cb)).unwrap();

        // join the same doc from node_1
        let doc_1 = node_1.doc_join(ticket).unwrap();

        // create author on node_1
        let author = node_1.author_create().unwrap();
        doc_1
            .set_bytes(author, b"hello".to_vec(), b"world".to_vec())
            .unwrap();
        let hash = found_r.recv().unwrap().unwrap();
        let val = node_1.blobs_read_to_bytes(hash.into()).unwrap();
        assert_eq!(b"world".to_vec(), val);
    }

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
        let derp_url = Arc::new(Url::from_string("https://derp.url").unwrap());
        //
        // create a NodeAddr
        let addrs = vec![Arc::new(ipv4), Arc::new(ipv6)];
        let expect_addrs = addrs.clone();
        let node_addr = NodeAddr::new(node_id.into(), Some(derp_url), addrs);
        //
        // test we have returned the expected addresses
        let got_addrs = node_addr.direct_addresses();
        let addrs = expect_addrs.iter().zip(got_addrs.iter());
        for (expect, got) in addrs {
            assert!(got.equal(expect.clone()));
            assert!(expect.equal(got.clone()));
        }

        let got_derp_url = node_addr.derp_url().unwrap();
        assert!(derp_url.equal(got_derp_url));
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
        let mut opts = QueryOptions::default();
        opts.offset = 10;
        opts.limit = 10;
        // all
        let all = Query::all(Some(opts));
        assert_eq!(10, all.offset());
        assert_eq!(Some(10), all.limit());

        let mut opts = QueryOptions::default();
        opts.direction = SortDirection::Desc;
        let single_latest_per_key = Query::single_latest_per_key(Some(opts));
        assert_eq!(0, single_latest_per_key.offset());
        assert_eq!(None, single_latest_per_key.limit());

        let mut opts = QueryOptions::default();
        opts.offset = 100;
        let author = Query::author(
            Arc::new(
                AuthorId::from_string(
                    "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q".to_string(),
                )
                .unwrap(),
            ),
            Some(opts),
        );
        assert_eq!(100, author.offset());
        assert_eq!(None, author.limit());

        let mut opts = QueryOptions::default();
        opts.limit = 100;
        let key_exact = Query::key_exact(b"key".to_vec(), Some(opts));
        assert_eq!(0, key_exact.offset());
        assert_eq!(Some(100), key_exact.limit());

        let opts = QueryOptions {
            sort_by: SortBy::KeyAuthor,
            direction: SortDirection::Desc,
            offset: 0,
            limit: 100,
        };
        let key_prefix = Query::key_prefix(b"prefix".to_vec(), Some(opts));
        assert_eq!(0, key_prefix.offset());
        assert_eq!(Some(100), key_prefix.limit());
    }
    #[test]
    fn test_doc_entry_basics() {
        let path = tempfile::tempdir().unwrap();
        let node = crate::IrohNode::new(path.path().to_string_lossy().into_owned()).unwrap();

        // create doc  and author
        let doc = node.doc_create().unwrap();
        let author = node.author_create().unwrap();

        // add entry
        let val = b"hello world!".to_vec();
        let key = b"foo".to_vec();
        let hash = doc
            .set_bytes(author.clone(), key.clone(), val.clone())
            .unwrap();

        // get entry
        let query = Query::author_key_exact(author, key.clone());
        let entry = doc.get_one(query.into()).unwrap().unwrap();

        assert!(hash.equal(entry.content_hash()));

        let got_val = doc.read_to_bytes(entry.clone()).unwrap();
        assert_eq!(val, got_val);
        assert_eq!(val.len() as u64, entry.content_len());
    }
    #[test]
    fn test_doc_import_export() {
        // create temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let in_root = temp_dir.path().join("in");
        std::fs::create_dir_all(in_root.clone()).unwrap();

        let out_root = temp_dir.path().join("out");
        let path = in_root.join("test");

        let size = 100;
        let mut buf = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut buf);
        let mut file = std::fs::File::create(path.clone()).unwrap();
        file.write_all(&buf.clone()).unwrap();
        file.flush().unwrap();

        // spawn node
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = crate::IrohNode::new(iroh_dir.path().to_string_lossy().into_owned()).unwrap();

        // create doc & author
        let doc = node.doc_create().unwrap();
        let author = node.author_create().unwrap();

        // import file
        let path_str = path.to_string_lossy().into_owned();
        let in_root_str = in_root.to_string_lossy().into_owned();
        let key = crate::path_to_key(path_str.clone(), None, Some(in_root_str)).unwrap();
        doc.import_file(author.clone(), key.clone(), path_str, true, None)
            .unwrap();

        // export file
        let entry = doc
            .get_one(Query::author_key_exact(author, key).into())
            .unwrap()
            .unwrap();
        let key = entry.key().to_vec();
        let out_root_str = out_root.to_string_lossy().into_owned();
        let path = crate::key_to_path(key, None, Some(out_root_str)).unwrap();
        doc.export_file(entry, path.clone(), None).unwrap();

        let got_bytes = std::fs::read(path).unwrap();
        assert_eq!(buf, got_bytes);
    }
}
