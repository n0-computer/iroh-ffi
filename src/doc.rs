use std::{path::PathBuf, str::FromStr, sync::Arc, time::SystemTime};

use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::{
    ticket::AddrInfoOptions, AuthorId, CallbackError, DocTicket, Hash, Iroh, IrohError, PublicKey,
};

#[derive(Debug, uniffi::Enum)]
pub enum CapabilityKind {
    /// A writable replica.
    Write = 1,
    /// A readable replica.
    Read = 2,
}

impl From<iroh::docs::CapabilityKind> for CapabilityKind {
    fn from(value: iroh::docs::CapabilityKind) -> Self {
        match value {
            iroh::docs::CapabilityKind::Write => Self::Write,
            iroh::docs::CapabilityKind::Read => Self::Read,
        }
    }
}

/// Iroh docs client.
#[derive(uniffi::Object)]
pub struct Docs {
    node: Iroh,
}

#[uniffi::export]
impl Iroh {
    /// Access to docs specific funtionaliy.
    pub fn docs(&self) -> Docs {
        Docs { node: self.clone() }
    }
}

impl Docs {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[uniffi::export]
impl Docs {
    /// Create a new doc.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn create(&self) -> Result<Arc<Doc>, IrohError> {
        let doc = self.client().docs().create().await?;

        Ok(Arc::new(Doc { inner: doc }))
    }

    /// Join and sync with an already existing document.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn join(&self, ticket: &DocTicket) -> Result<Arc<Doc>, IrohError> {
        let doc = self.client().docs().import(ticket.clone().into()).await?;
        Ok(Arc::new(Doc { inner: doc }))
    }

    /// Join and sync with an already existing document and subscribe to events on that document.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn join_and_subscribe(
        &self,
        ticket: &DocTicket,
        cb: Arc<dyn SubscribeCallback>,
    ) -> Result<Arc<Doc>, IrohError> {
        let (doc, mut stream) = self
            .client()
            .docs()
            .import_and_subscribe(ticket.clone().into())
            .await?;

        tokio::spawn(async move {
            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => {
                        if let Err(err) = cb.event(Arc::new(event.into())).await {
                            println!("cb error: {:?}", err);
                        }
                    }
                    Err(err) => {
                        println!("rpc error: {:?}", err);
                    }
                }
            }
        });

        Ok(Arc::new(Doc { inner: doc }))
    }

    /// List all the docs we have access to on this node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn list(&self) -> Result<Vec<NamespaceAndCapability>, IrohError> {
        let docs = self
            .client()
            .docs()
            .list()
            .await?
            .map_ok(|(namespace, capability)| NamespaceAndCapability {
                namespace: namespace.to_string(),
                capability: capability.into(),
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(docs)
    }

    /// Get a [`Doc`].
    ///
    /// Returns None if the document cannot be found.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn open(&self, id: String) -> Result<Option<Arc<Doc>>, IrohError> {
        let namespace_id = iroh::docs::NamespaceId::from_str(&id)?;
        let doc = self.client().docs().open(namespace_id).await?;

        Ok(doc.map(|d| Arc::new(Doc { inner: d })))
    }

    /// Delete a document from the local node.
    ///
    /// This is a destructive operation. Both the document secret key and all entries in the
    /// document will be permanently deleted from the node's storage. Content blobs will be deleted
    /// through garbage collection unless they are referenced from another document or tag.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn drop_doc(&self, doc_id: String) -> Result<(), IrohError> {
        let doc_id = iroh::docs::NamespaceId::from_str(&doc_id)?;
        self.client()
            .docs()
            .drop_doc(doc_id)
            .await
            .map_err(IrohError::from)
    }
}

/// The namespace id and CapabilityKind (read/write) of the doc
#[derive(Debug, uniffi::Record)]
pub struct NamespaceAndCapability {
    /// The namespace id of the doc
    pub namespace: String,
    /// The capability you have for the doc (read/write)
    pub capability: CapabilityKind,
}

/// A representation of a mutable, synchronizable key-value store.
#[derive(Clone, uniffi::Object)]
pub struct Doc {
    pub(crate) inner: iroh::client::Doc,
}

#[uniffi::export]
impl Doc {
    /// Get the document id of this doc.
    #[uniffi::method]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    /// Close the document.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn close_me(&self) -> Result<(), IrohError> {
        self.inner.close().await.map_err(IrohError::from)
    }

    /// Set the content of a key to a byte array.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn set_bytes(
        &self,
        author_id: &AuthorId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Arc<Hash>, IrohError> {
        let hash = self.inner.set_bytes(author_id.0, key, value).await?;
        Ok(Arc::new(Hash(hash)))
    }

    /// Set an entries on the doc via its key, hash, and size.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn set_hash(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        hash: Arc<Hash>,
        size: u64,
    ) -> Result<(), IrohError> {
        self.inner.set_hash(author_id.0, key, hash.0, size).await?;
        Ok(())
    }

    /// Add an entry from an absolute file path
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn import_file(
        &self,
        author: Arc<AuthorId>,
        key: Vec<u8>,
        path: String,
        in_place: bool,
        cb: Option<Arc<dyn DocImportFileCallback>>,
    ) -> Result<(), IrohError> {
        let mut stream = self
            .inner
            .import_file(author.0, Bytes::from(key), PathBuf::from(path), in_place)
            .await?;

        while let Some(progress) = stream.next().await {
            let progress = progress?;
            if let Some(ref cb) = cb {
                cb.progress(Arc::new(progress.into())).await?;
            }
        }
        Ok(())
    }

    /// Export an entry as a file to a given absolute path
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn export_file(
        &self,
        entry: Arc<Entry>,
        path: String,
        cb: Option<Arc<dyn DocExportFileCallback>>,
    ) -> Result<(), IrohError> {
        let mut stream = self
            .inner
            .export_file(
                entry.0.clone(),
                std::path::PathBuf::from(path),
                // TODO(b5) - plumb up the export mode, currently it's always copy
                iroh::blobs::store::ExportMode::Copy,
            )
            .await?;
        while let Some(progress) = stream.next().await {
            let progress = progress?;
            if let Some(ref cb) = cb {
                cb.progress(Arc::new(progress.into())).await?;
            }
        }
        Ok(())
    }

    /// Delete entries that match the given `author` and key `prefix`.
    ///
    /// This inserts an empty entry with the key set to `prefix`, effectively clearing all other
    /// entries whose key starts with or is equal to the given `prefix`.
    ///
    /// Returns the number of entries deleted.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn delete(
        &self,
        author_id: Arc<AuthorId>,
        prefix: Vec<u8>,
    ) -> Result<u64, IrohError> {
        let num_del = self.inner.del(author_id.0, prefix).await?;

        u64::try_from(num_del).map_err(|e| anyhow::Error::from(e).into())
    }

    /// Get an entry for a key and author.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_exact(
        &self,
        author: Arc<AuthorId>,
        key: Vec<u8>,
        include_empty: bool,
    ) -> Result<Option<Arc<Entry>>, IrohError> {
        self.inner
            .get_exact(author.0, key, include_empty)
            .await
            .map(|e| e.map(|e| Arc::new(e.into())))
            .map_err(IrohError::from)
    }

    /// Get entries.
    ///
    /// Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_many(&self, query: Arc<Query>) -> Result<Vec<Arc<Entry>>, IrohError> {
        let entries = self
            .inner
            .get_many(query.0.clone())
            .await?
            .map_ok(|e| Arc::new(Entry(e)))
            .try_collect::<Vec<_>>()
            .await?;
        Ok(entries)
    }

    /// Get the latest entry for a key and author.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_one(&self, query: Arc<Query>) -> Result<Option<Arc<Entry>>, IrohError> {
        let res = self
            .inner
            .get_one((*query).clone().0)
            .await
            .map(|e| e.map(|e| Arc::new(e.into())))?;
        Ok(res)
    }

    /// Share this document with peers over a ticket.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn share(
        &self,
        mode: ShareMode,
        addr_options: AddrInfoOptions,
    ) -> Result<Arc<DocTicket>, IrohError> {
        let res = self
            .inner
            .share(mode.into(), addr_options.into())
            .await
            .map(|ticket| Arc::new(ticket.into()))?;
        Ok(res)
    }

    /// Start to sync this document with a list of peers.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn start_sync(&self, peers: Vec<Arc<NodeAddr>>) -> Result<(), IrohError> {
        self.inner
            .start_sync(
                peers
                    .into_iter()
                    .map(|p| (*p).clone().try_into())
                    .collect::<Result<Vec<_>, IrohError>>()?,
            )
            .await?;
        Ok(())
    }

    /// Stop the live sync for this document.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn leave(&self) -> Result<(), IrohError> {
        self.inner.leave().await?;
        Ok(())
    }

    /// Subscribe to events for this document.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn subscribe(&self, cb: Arc<dyn SubscribeCallback>) -> Result<(), IrohError> {
        let client = self.inner.clone();
        tokio::task::spawn(async move {
            let mut sub = client.subscribe().await.unwrap();
            while let Some(event) = sub.next().await {
                match event {
                    Ok(event) => {
                        if let Err(err) = cb.event(Arc::new(event.into())).await {
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

    /// Get status info for this document
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn status(&self) -> Result<OpenState, IrohError> {
        let res = self.inner.status().await.map(|o| o.into())?;
        Ok(res)
    }

    /// Set the download policy for this document
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn set_download_policy(&self, policy: Arc<DownloadPolicy>) -> Result<(), IrohError> {
        self.inner
            .set_download_policy((*policy).clone().into())
            .await?;
        Ok(())
    }

    /// Get the download policy for this document
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_download_policy(&self) -> Result<Arc<DownloadPolicy>, IrohError> {
        let res = self
            .inner
            .get_download_policy()
            .await
            .map(|policy| Arc::new(policy.into()))?;
        Ok(res)
    }

    /// Get sync peers for this document
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_sync_peers(&self) -> Result<Option<Vec<Vec<u8>>>, IrohError> {
        let list = self.inner.get_sync_peers().await?;
        let list = list.map(|l| l.into_iter().map(|p| p.to_vec()).collect());
        Ok(list)
    }
}

/// Download policy to decide which content blobs shall be downloaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
pub enum DownloadPolicy {
    /// Do not download any key unless it matches one of the filters.
    NothingExcept(Vec<Arc<FilterKind>>),
    /// Download every key unless it matches one of the filters.
    EverythingExcept(Vec<Arc<FilterKind>>),
}

#[uniffi::export]
impl DownloadPolicy {
    /// Download everything
    #[uniffi::constructor]
    pub fn everything() -> Self {
        DownloadPolicy::EverythingExcept(Vec::default())
    }

    /// Download nothing
    #[uniffi::constructor]
    pub fn nothing() -> Self {
        DownloadPolicy::NothingExcept(Vec::default())
    }

    /// Download nothing except keys that match the given filters

    #[uniffi::constructor]
    pub fn nothing_except(filters: Vec<Arc<FilterKind>>) -> Self {
        DownloadPolicy::NothingExcept(filters)
    }

    /// Download everything except keys that match the given filters
    #[uniffi::constructor]
    pub fn everything_except(filters: Vec<Arc<FilterKind>>) -> Self {
        DownloadPolicy::EverythingExcept(filters)
    }
}

impl From<iroh::docs::store::DownloadPolicy> for DownloadPolicy {
    fn from(value: iroh::docs::store::DownloadPolicy) -> Self {
        match value {
            iroh::docs::store::DownloadPolicy::NothingExcept(filters) => {
                DownloadPolicy::NothingExcept(
                    filters.into_iter().map(|f| Arc::new(f.into())).collect(),
                )
            }
            iroh::docs::store::DownloadPolicy::EverythingExcept(filters) => {
                DownloadPolicy::EverythingExcept(
                    filters.into_iter().map(|f| Arc::new(f.into())).collect(),
                )
            }
        }
    }
}

impl From<DownloadPolicy> for iroh::docs::store::DownloadPolicy {
    fn from(value: DownloadPolicy) -> Self {
        match value {
            DownloadPolicy::NothingExcept(filters) => {
                iroh::docs::store::DownloadPolicy::NothingExcept(
                    filters.into_iter().map(|f| f.0.clone()).collect(),
                )
            }
            DownloadPolicy::EverythingExcept(filters) => {
                iroh::docs::store::DownloadPolicy::EverythingExcept(
                    filters.into_iter().map(|f| f.0.clone()).collect(),
                )
            }
        }
    }
}

/// Filter strategy used in download policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
pub struct FilterKind(pub(crate) iroh::docs::store::FilterKind);

#[uniffi::export]
impl FilterKind {
    /// Verifies whether this filter matches a given key
    pub fn matches(&self, key: Vec<u8>) -> bool {
        self.0.matches(key)
    }

    /// Returns a FilterKind that matches if the contained bytes are a prefix of the key.
    #[uniffi::constructor]
    pub fn prefix(prefix: Vec<u8>) -> FilterKind {
        FilterKind(iroh::docs::store::FilterKind::Prefix(Bytes::from(prefix)))
    }

    /// Returns a FilterKind that matches if the contained bytes and the key are the same.
    #[uniffi::constructor]
    pub fn exact(key: Vec<u8>) -> FilterKind {
        FilterKind(iroh::docs::store::FilterKind::Exact(Bytes::from(key)))
    }
}

impl From<iroh::docs::store::FilterKind> for FilterKind {
    fn from(value: iroh::docs::store::FilterKind) -> Self {
        FilterKind(value)
    }
}

/// The state for an open replica.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, uniffi::Record)]
pub struct OpenState {
    /// Whether to accept sync requests for this replica.
    pub sync: bool,
    /// How many event subscriptions are open
    pub subscribers: u64,
    /// By how many handles the replica is currently held open
    pub handles: u64,
}

impl From<iroh::docs::actor::OpenState> for OpenState {
    fn from(value: iroh::docs::actor::OpenState) -> Self {
        OpenState {
            sync: value.sync,
            subscribers: value.subscribers as u64,
            handles: value.handles as u64,
        }
    }
}

/// A peer and it's addressing information.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct NodeAddr {
    node_id: Arc<PublicKey>,
    relay_url: Option<String>,
    addresses: Vec<String>,
}

#[uniffi::export]
impl NodeAddr {
    /// Create a new [`NodeAddr`] with empty [`AddrInfo`].
    #[uniffi::constructor]
    pub fn new(node_id: &PublicKey, derp_url: Option<String>, addresses: Vec<String>) -> Self {
        Self {
            node_id: Arc::new(node_id.clone()),
            relay_url: derp_url,
            addresses,
        }
    }

    /// Get the direct addresses of this peer.
    pub fn direct_addresses(&self) -> Vec<String> {
        self.addresses.clone()
    }

    /// Get the home relay URL for this peer
    pub fn relay_url(&self) -> Option<String> {
        self.relay_url.clone()
    }

    /// Returns true if both NodeAddr's have the same values
    pub fn equal(&self, other: &NodeAddr) -> bool {
        self == other
    }
}

impl TryFrom<NodeAddr> for iroh::net::endpoint::NodeAddr {
    type Error = IrohError;
    fn try_from(value: NodeAddr) -> Result<Self, Self::Error> {
        let mut node_addr = iroh::net::endpoint::NodeAddr::new((&*value.node_id).into());
        let addresses = value
            .direct_addresses()
            .into_iter()
            .map(|addr| {
                std::net::SocketAddr::from_str(&addr).map_err(|e| anyhow::Error::from(e).into())
            })
            .collect::<Result<Vec<_>, IrohError>>()?;

        if let Some(derp_url) = value.relay_url() {
            let url = url::Url::parse(&derp_url).map_err(anyhow::Error::from)?;

            node_addr = node_addr.with_relay_url(url.into());
        }
        node_addr = node_addr.with_direct_addresses(addresses);
        Ok(node_addr)
    }
}

impl From<iroh::net::endpoint::NodeAddr> for NodeAddr {
    fn from(value: iroh::net::endpoint::NodeAddr) -> Self {
        NodeAddr {
            node_id: Arc::new(value.node_id.into()),
            relay_url: value.info.relay_url.map(|url| url.to_string()),
            addresses: value
                .info
                .direct_addresses
                .into_iter()
                .map(|d| d.to_string())
                .collect(),
        }
    }
}

/// Intended capability for document share tickets
#[derive(Debug, uniffi::Enum)]
pub enum ShareMode {
    /// Read-only access
    Read,
    /// Write access
    Write,
}

impl From<ShareMode> for iroh::client::docs::ShareMode {
    fn from(mode: ShareMode) -> Self {
        match mode {
            ShareMode::Read => iroh::client::docs::ShareMode::Read,
            ShareMode::Write => iroh::client::docs::ShareMode::Write,
        }
    }
}

/// A single entry in a [`Doc`]
///
/// An entry is identified by a key, its [`AuthorId`], and the [`Doc`]'s
/// namespace id. Its value is the 32-byte BLAKE3 [`hash`]
/// of the entry's content data, the size of this content data, and a timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Object)]
pub struct Entry(pub(crate) iroh::client::docs::Entry);

impl From<iroh::client::docs::Entry> for Entry {
    fn from(e: iroh::client::docs::Entry) -> Self {
        Entry(e)
    }
}

#[uniffi::export]
impl Entry {
    /// Get the [`AuthorId`] of this entry.
    #[uniffi::method]
    pub fn author(&self) -> Arc<AuthorId> {
        Arc::new(AuthorId(self.0.id().author()))
    }

    /// Get the content_hash of this entry.
    #[uniffi::method]
    pub fn content_hash(&self) -> Arc<Hash> {
        Arc::new(Hash(self.0.content_hash()))
    }

    /// Get the content_length of this entry.
    #[uniffi::method]
    pub fn content_len(&self) -> u64 {
        self.0.content_len()
    }

    /// Get the key of this entry.
    #[uniffi::method]
    pub fn key(&self) -> Vec<u8> {
        self.0.id().key().to_vec()
    }

    /// Get the namespace id of this entry.
    #[uniffi::method]
    pub fn namespace(&self) -> String {
        self.0.id().namespace().to_string()
    }

    /// Get the timestamp when this entry was written.
    #[uniffi::method]
    pub fn timestamp(&self) -> u64 {
        self.0.timestamp()
    }

    /// Read all content of an [`Entry`] into a buffer.
    /// This allocates a buffer for the full entry. Use only if you know that the entry you're
    /// reading is small. If not sure, use [`Self::content_len`] and check the size with
    /// before calling [`Self::content_bytes`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn content_bytes(&self, doc: Arc<Doc>) -> Result<Vec<u8>, IrohError> {
        let res = self.0.content_bytes(&doc.inner).await.map(|c| c.to_vec())?;
        Ok(res)
    }
}

///d Fields by which the query can be sorted
#[derive(Clone, Debug, Default, Serialize, Deserialize, uniffi::Enum)]
pub enum SortBy {
    /// Sort by key, then author.
    KeyAuthor,
    /// Sort by author, then key.
    #[default]
    AuthorKey,
}

impl From<iroh::docs::store::SortBy> for SortBy {
    fn from(value: iroh::docs::store::SortBy) -> Self {
        match value {
            iroh::docs::store::SortBy::AuthorKey => SortBy::AuthorKey,
            iroh::docs::store::SortBy::KeyAuthor => SortBy::KeyAuthor,
        }
    }
}

impl From<SortBy> for iroh::docs::store::SortBy {
    fn from(value: SortBy) -> Self {
        match value {
            SortBy::AuthorKey => iroh::docs::store::SortBy::AuthorKey,
            SortBy::KeyAuthor => iroh::docs::store::SortBy::KeyAuthor,
        }
    }
}

/// Sort direction
#[derive(Clone, Debug, Default, Serialize, Deserialize, uniffi::Enum)]
pub enum SortDirection {
    /// Sort ascending
    #[default]
    Asc,
    /// Sort descending
    Desc,
}

impl From<iroh::docs::store::SortDirection> for SortDirection {
    fn from(value: iroh::docs::store::SortDirection) -> Self {
        match value {
            iroh::docs::store::SortDirection::Asc => SortDirection::Asc,
            iroh::docs::store::SortDirection::Desc => SortDirection::Desc,
        }
    }
}

impl From<SortDirection> for iroh::docs::store::SortDirection {
    fn from(value: SortDirection) -> Self {
        match value {
            SortDirection::Asc => iroh::docs::store::SortDirection::Asc,
            SortDirection::Desc => iroh::docs::store::SortDirection::Desc,
        }
    }
}

/// Build a Query to search for an entry or entries in a doc.
///
/// Use this with `QueryOptions` to determine sorting, grouping, and pagination.
#[derive(Clone, Debug, uniffi::Object)]
pub struct Query(pub(crate) iroh::docs::store::Query);

/// Options for sorting and pagination for using [`Query`]s.
#[derive(Clone, Debug, Default, uniffi::Record)]
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

#[uniffi::export]
impl Query {
    /// Query all records.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[uniffi::constructor]
    pub fn all(opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::docs::store::Query::all();

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by.into(), opts.direction.into());
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
    #[uniffi::constructor]
    pub fn single_latest_per_key(opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::docs::store::Query::single_latest_per_key();

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_direction(opts.direction.into());
        }
        Query(builder.build())
    }

    /// Query exactly the key, but only the latest entry for it, omitting older entries if the entry was written
    /// to by multiple authors.
    #[uniffi::constructor]
    pub fn single_latest_per_key_exact(key: Vec<u8>) -> Self {
        let builder = iroh::docs::store::Query::single_latest_per_key()
            .key_exact(key)
            .build();
        Query(builder)
    }

    /// Query only the latest entry for each key, with this prefix, omitting older entries if the entry was written
    /// to by multiple authors.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[uniffi::constructor]
    pub fn single_latest_per_key_prefix(prefix: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::docs::store::Query::single_latest_per_key().key_prefix(prefix);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
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
    #[uniffi::constructor]
    pub fn author(author: &AuthorId, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::docs::store::Query::author(author.0);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by.into(), opts.direction.into());
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
    #[uniffi::constructor]
    pub fn key_exact(key: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::docs::store::Query::key_exact(key);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by.into(), opts.direction.into());
        }
        Query(builder.build())
    }

    /// Create a Query for a single key and author.
    #[uniffi::constructor]
    pub fn author_key_exact(author: &AuthorId, key: Vec<u8>) -> Self {
        let builder = iroh::docs::store::Query::author(author.0).key_exact(key);
        Query(builder.build())
    }

    /// Create a query for all entries with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[uniffi::constructor]
    pub fn key_prefix(prefix: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::docs::store::Query::key_prefix(prefix);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by.into(), opts.direction.into());
        }
        Query(builder.build())
    }

    /// Create a query for all entries of a single author with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[uniffi::constructor]
    pub fn author_key_prefix(
        author: &AuthorId,
        prefix: Vec<u8>,
        opts: Option<QueryOptions>,
    ) -> Self {
        let mut builder = iroh::docs::store::Query::author(author.0).key_prefix(prefix);

        if let Some(opts) = opts {
            if opts.offset != 0 {
                builder = builder.offset(opts.offset);
            }
            if opts.limit != 0 {
                builder = builder.limit(opts.limit);
            }
            builder = builder.sort_by(opts.sort_by.into(), opts.direction.into());
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

/// The `progress` method will be called for each `SubscribeProgress` event that is
/// emitted during a `node.doc_subscribe`. Use the `SubscribeProgress.type()`
/// method to check the `LiveEvent`
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait SubscribeCallback: Send + Sync + 'static {
    async fn event(&self, event: Arc<LiveEvent>) -> Result<(), CallbackError>;
}

/// Events informing about actions of the live sync progress
#[derive(Debug, Serialize, Deserialize, uniffi::Object)]
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
    /// All pending content is now ready.
    ///
    /// This event signals that all queued content downloads from the last sync run have either
    /// completed or failed.
    ///
    /// It will only be emitted after a [`Self::SyncFinished`] event, never before.
    ///
    /// Receiving this event does not guarantee that all content in the document is available. If
    /// blobs failed to download, this event will still be emitted after all operations completed.
    PendingContentReady,
}

/// The type of events that can be emitted during the live sync progress
#[derive(Debug, uniffi::Enum)]
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
    /// All pending content is now ready.
    ///
    /// This event signals that all queued content downloads from the last sync run have either
    /// completed or failed.
    ///
    /// It will only be emitted after a [`Self::SyncFinished`] event, never before.
    ///
    /// Receiving this event does not guarantee that all content in the document is available. If
    /// blobs failed to download, this event will still be emitted after all operations completed.
    PendingContentReady,
}

#[uniffi::export]
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
            Self::PendingContentReady => LiveEventType::PendingContentReady,
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

impl From<iroh::client::docs::LiveEvent> for LiveEvent {
    fn from(value: iroh::client::docs::LiveEvent) -> Self {
        match value {
            iroh::client::docs::LiveEvent::InsertLocal { entry } => LiveEvent::InsertLocal {
                entry: entry.into(),
            },
            iroh::client::docs::LiveEvent::InsertRemote {
                from,
                entry,
                content_status,
            } => LiveEvent::InsertRemote {
                from: from.into(),
                entry: entry.into(),
                content_status: content_status.into(),
            },
            iroh::client::docs::LiveEvent::ContentReady { hash } => {
                LiveEvent::ContentReady { hash: hash.into() }
            }
            iroh::client::docs::LiveEvent::NeighborUp(key) => LiveEvent::NeighborUp(key.into()),
            iroh::client::docs::LiveEvent::NeighborDown(key) => LiveEvent::NeighborDown(key.into()),
            iroh::client::docs::LiveEvent::SyncFinished(e) => LiveEvent::SyncFinished(e.into()),
            iroh::client::docs::LiveEvent::PendingContentReady => LiveEvent::PendingContentReady,
        }
    }
}

/// Outcome of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
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

impl From<iroh::client::docs::SyncEvent> for SyncEvent {
    fn from(value: iroh::client::docs::SyncEvent) -> Self {
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
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Copy, uniffi::Enum)]
pub enum SyncReason {
    /// Direct join request via API
    DirectJoin,
    /// Peer showed up as new neighbor in the gossip swarm
    NewNeighbor,
    /// We synced after receiving a sync report that indicated news for us
    SyncReport,
    /// We received a sync report while a sync was running, so run again afterwars
    Resync,
}

impl From<iroh::client::docs::SyncReason> for SyncReason {
    fn from(value: iroh::client::docs::SyncReason) -> Self {
        match value {
            iroh::client::docs::SyncReason::DirectJoin => Self::DirectJoin,
            iroh::client::docs::SyncReason::NewNeighbor => Self::NewNeighbor,
            iroh::client::docs::SyncReason::SyncReport => Self::SyncReport,
            iroh::client::docs::SyncReason::Resync => Self::Resync,
        }
    }
}

/// Why we performed a sync exchange
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Enum)]
pub enum Origin {
    /// public, use a unit variant
    Connect { reason: SyncReason },
    /// A peer connected to us and we accepted the exchange
    Accept,
}

impl From<iroh::client::docs::Origin> for Origin {
    fn from(value: iroh::client::docs::Origin) -> Self {
        match value {
            iroh::client::docs::Origin::Connect(reason) => Self::Connect {
                reason: reason.into(),
            },
            iroh::client::docs::Origin::Accept => Self::Accept,
        }
    }
}

/// Outcome of an InsertRemove event.
#[derive(Debug, Serialize, Deserialize, uniffi::Record)]
pub struct InsertRemoteEvent {
    /// The peer that sent us the entry.
    pub from: Arc<PublicKey>,
    /// The inserted entry.
    pub entry: Arc<Entry>,
    /// If the content is available at the local node
    pub content_status: ContentStatus,
}

/// Whether the content status is available on a node.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Enum)]
pub enum ContentStatus {
    /// The content is completely available.
    Complete,
    /// The content is partially available.
    Incomplete,
    /// The content is missing.
    Missing,
}

impl From<iroh::docs::ContentStatus> for ContentStatus {
    fn from(value: iroh::docs::ContentStatus) -> Self {
        match value {
            iroh::docs::ContentStatus::Complete => Self::Complete,
            iroh::docs::ContentStatus::Incomplete => Self::Incomplete,
            iroh::docs::ContentStatus::Missing => Self::Missing,
        }
    }
}

/// The `progress` method will be called for each `DocImportProgress` event that is
/// emitted during a `doc.import_file()` call. Use the `DocImportProgress.type()`
/// method to check the `DocImportProgressType`
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait DocImportFileCallback: Send + Sync + 'static {
    async fn progress(&self, progress: Arc<DocImportProgress>) -> Result<(), CallbackError>;
}

/// The type of `DocImportProgress` event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocImportProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// A DocImportProgress event indicating we've made progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocImportProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DocImportProgress event indicating we are finished adding `id` to the data store and the hash is `hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocImportProgressIngestDone {
    /// The unique id of the entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// A DocImportProgress event indicating we are done setting the entry to the doc
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocImportProgressAllDone {
    /// The key of the entry
    pub key: Vec<u8>,
}

/// A DocImportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocImportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
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

impl From<iroh::client::docs::ImportProgress> for DocImportProgress {
    fn from(value: iroh::client::docs::ImportProgress) -> Self {
        match value {
            iroh::client::docs::ImportProgress::Found { id, name, size } => {
                DocImportProgress::Found(DocImportProgressFound { id, name, size })
            }
            iroh::client::docs::ImportProgress::Progress { id, offset } => {
                DocImportProgress::Progress(DocImportProgressProgress { id, offset })
            }
            iroh::client::docs::ImportProgress::IngestDone { id, hash } => {
                DocImportProgress::IngestDone(DocImportProgressIngestDone {
                    id,
                    hash: Arc::new(hash.into()),
                })
            }
            iroh::client::docs::ImportProgress::AllDone { key } => {
                DocImportProgress::AllDone(DocImportProgressAllDone { key: key.into() })
            }
            iroh::client::docs::ImportProgress::Abort(err) => {
                DocImportProgress::Abort(DocImportProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

#[uniffi::export]
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
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait DocExportFileCallback: Send + Sync + 'static {
    async fn progress(&self, progress: Arc<DocExportProgress>) -> Result<(), CallbackError>;
}

/// The type of `DocExportProgress` event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum DocExportProgressType {
    /// An item was found with name `name`, from now on referred to via `id`
    Found,
    /// We got progress exporting item `id`.
    Progress,
    /// We finished exporting a blob with `id`
    Done,
    /// We are done writing the entry to the filesystem
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort,
}

/// A DocExportProgress event indicating a file was found with name `name`, from now on referred to via `id`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocExportProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
    /// The size of the entry in bytes.
    pub size: u64,
    /// The path where we are writing the entry
    pub outpath: String,
}

/// A DocExportProgress event indicating we've made progress exporting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocExportProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DocExportProgress event indicating a single blob wit `id` is done
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocExportProgressDone {
    /// The unique id of the entry.
    pub id: u64,
}

/// A DocExportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DocExportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
pub enum DocExportProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    Found(DocExportProgressFound),
    /// We got progress ingesting item `id`.
    Progress(DocExportProgressProgress),
    /// We finished exporting a blob
    Done(DocExportProgressDone),
    /// We are done with the whole operation.
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort(DocExportProgressAbort),
}

impl From<iroh::blobs::export::ExportProgress> for DocExportProgress {
    fn from(value: iroh::blobs::export::ExportProgress) -> Self {
        match value {
            iroh::blobs::export::ExportProgress::Found {
                id,
                hash,
                size,
                outpath,
                // TODO (b5) - currently ignoring meta field. meta is probably the key of the entry that's being exported
                ..
            } => DocExportProgress::Found(DocExportProgressFound {
                id,
                hash: Arc::new(hash.into()),
                // TODO(b5) - this is ignoring verification status of file size!
                size: size.value(),
                outpath: outpath.to_string_lossy().to_string(),
            }),
            iroh::blobs::export::ExportProgress::Progress { id, offset } => {
                DocExportProgress::Progress(DocExportProgressProgress { id, offset })
            }
            iroh::blobs::export::ExportProgress::Done { id } => {
                DocExportProgress::Done(DocExportProgressDone { id })
            }
            iroh::blobs::export::ExportProgress::AllDone => DocExportProgress::AllDone,
            iroh::blobs::export::ExportProgress::Abort(err) => {
                DocExportProgress::Abort(DocExportProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

#[uniffi::export]
impl DocExportProgress {
    /// Get the type of event
    pub fn r#type(&self) -> DocExportProgressType {
        match self {
            DocExportProgress::Found(_) => DocExportProgressType::Found,
            DocExportProgress::Progress(_) => DocExportProgressType::Progress,
            DocExportProgress::Done(_) => DocExportProgressType::Done,
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
    use crate::PublicKey;
    use rand::RngCore;
    use tokio::{io::AsyncWriteExt, sync::mpsc};

    #[tokio::test]
    async fn test_doc_create() {
        let path = tempfile::tempdir().unwrap();
        let node = Iroh::persistent(
            path.path()
                .join("doc-create")
                .to_string_lossy()
                .into_owned(),
        )
        .await
        .unwrap();
        let node_id = node.net().node_id().await.unwrap();
        println!("id: {}", node_id);
        let doc = node.docs().create().await.unwrap();
        let doc_id = doc.id();
        println!("doc_id: {}", doc_id);

        let doc_ticket = doc
            .share(crate::doc::ShareMode::Write, AddrInfoOptions::Id)
            .await
            .unwrap();
        println!("doc_ticket: {}", doc_ticket);
        node.docs().join(&doc_ticket).await.unwrap();
    }

    #[tokio::test]
    async fn test_basic_sync() {
        // create node_0
        let iroh_dir = tempfile::tempdir().unwrap();

        let node_0 = Iroh::persistent(
            iroh_dir
                .path()
                .join("basic-sync-0")
                .to_string_lossy()
                .into_owned(),
        )
        .await
        .unwrap();

        // create node_1
        let node_1 = Iroh::persistent(
            iroh_dir
                .path()
                .join("basic-sync-1")
                .to_string_lossy()
                .into_owned(),
        )
        .await
        .unwrap();

        // create doc on node_0
        let doc_0 = node_0.docs().create().await.unwrap();
        let ticket = doc_0
            .share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses)
            .await
            .unwrap();

        // subscribe to sync events
        let (found_s, mut found_r) = mpsc::channel(8);
        struct Callback {
            found_s: mpsc::Sender<Arc<LiveEvent>>,
        }
        #[async_trait::async_trait]
        impl SubscribeCallback for Callback {
            async fn event(&self, event: Arc<LiveEvent>) -> Result<(), CallbackError> {
                println!("event {:?}", event);
                self.found_s.send(event).await.unwrap();
                Ok(())
            }
        }
        let cb_0 = Callback { found_s };
        doc_0.subscribe(Arc::new(cb_0)).await.unwrap();

        // join the same doc from node_1
        let (found_s_1, mut found_r_1) = mpsc::channel(8);
        let cb_1 = Callback { found_s: found_s_1 };
        let doc_1 = node_1
            .docs()
            .join_and_subscribe(&ticket, Arc::new(cb_1))
            .await
            .unwrap();

        // wait for initial sync to be one
        while let Some(event) = found_r_1.recv().await {
            if let LiveEvent::SyncFinished(_) = *event {
                break;
            }
        }

        // create author on node_1
        let author = node_1.authors().create().await.unwrap();
        doc_1
            .set_bytes(&author, b"hello".to_vec(), b"world".to_vec())
            .await
            .unwrap();
        while let Some(event) = found_r.recv().await {
            if let LiveEvent::ContentReady { ref hash } = *event {
                let val = node_1
                    .blobs()
                    .read_to_bytes(hash.clone().into())
                    .await
                    .unwrap();
                assert_eq!(b"world".to_vec(), val);
                break;
            }
        }
    }

    #[test]
    fn test_node_addr() {
        //
        // create a node_id
        let key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva";
        let node_id = PublicKey::from_string(key_str.into()).unwrap();
        //
        // create socketaddrs
        let port = 3000;
        let ipv4 = format!("127.0.0.1:{port}");
        let ipv6 = format!("::1:{port}");
        //
        // derp region
        let derp_url = String::from("https://derp.url");
        //
        // create a NodeAddr
        let addrs = vec![ipv4, ipv6];
        let expect_addrs = addrs.clone();
        let node_addr = NodeAddr::new(&node_id, Some(derp_url.clone()), addrs);
        //
        // test we have returned the expected addresses
        let got_addrs = node_addr.direct_addresses();
        let addrs = expect_addrs.iter().zip(got_addrs.iter());
        for (expect, got) in addrs {
            assert_eq!(got, expect);
        }

        let got_derp_url = node_addr.relay_url().unwrap();
        assert_eq!(derp_url, got_derp_url);
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
        assert!(author.equal(&author_0));
        assert!(author_0.equal(&author));
    }

    #[test]
    fn test_query() {
        let opts = QueryOptions {
            offset: 10,
            limit: 10,
            ..QueryOptions::default()
        };
        // all
        let all = Query::all(Some(opts));
        assert_eq!(10, all.offset());
        assert_eq!(Some(10), all.limit());

        let opts = QueryOptions {
            direction: SortDirection::Desc,
            ..QueryOptions::default()
        };
        let single_latest_per_key = Query::single_latest_per_key(Some(opts));
        assert_eq!(0, single_latest_per_key.offset());
        assert_eq!(None, single_latest_per_key.limit());

        let opts = QueryOptions {
            offset: 100,
            ..QueryOptions::default()
        };
        let author = Query::author(
            &AuthorId::from_string(
                "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q".to_string(),
            )
            .unwrap(),
            Some(opts),
        );
        assert_eq!(100, author.offset());
        assert_eq!(None, author.limit());

        let opts = QueryOptions {
            limit: 100,
            ..QueryOptions::default()
        };
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

    #[tokio::test]
    async fn test_doc_entry_basics() {
        let path = tempfile::tempdir().unwrap();
        let node = crate::Iroh::persistent(
            path.path()
                .join("doc-entry-basics")
                .to_string_lossy()
                .into_owned(),
        )
        .await
        .unwrap();

        // create doc  and author
        let doc = node.docs().create().await.unwrap();
        let author = node.authors().create().await.unwrap();

        // add entry
        let val = b"hello world!".to_vec();
        let key = b"foo".to_vec();
        let hash = doc
            .set_bytes(&author, key.clone(), val.clone())
            .await
            .unwrap();

        // get entry
        let query = Query::author_key_exact(&author, key.clone());
        let entry = doc.get_one(query.into()).await.unwrap().unwrap();

        assert!(hash.equal(&entry.content_hash()));

        let got_val = entry.content_bytes(doc).await.unwrap();
        assert_eq!(val, got_val);
        assert_eq!(val.len() as u64, entry.content_len());
    }

    #[tokio::test]
    async fn test_doc_import_export() {
        // create temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let in_root = temp_dir.path().join("import-export-in");
        tokio::fs::create_dir_all(in_root.clone()).await.unwrap();

        let out_root = temp_dir.path().join("import-export-out");
        let path = in_root.join("test");

        let size = 100;
        let mut buf = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut buf);
        let mut file = tokio::fs::File::create(path.clone()).await.unwrap();
        file.write_all(&buf.clone()).await.unwrap();
        file.flush().await.unwrap();

        // spawn node
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = crate::Iroh::persistent(
            iroh_dir
                .path()
                .join("import-export-node")
                .to_string_lossy()
                .into_owned(),
        )
        .await
        .unwrap();

        // create doc & author
        let doc = node.docs().create().await.unwrap();
        let author = node.authors().create().await.unwrap();

        // import file
        let path_str = path.to_string_lossy().into_owned();
        let in_root_str = in_root.to_string_lossy().into_owned();
        let key = crate::path_to_key(path_str.clone(), None, Some(in_root_str)).unwrap();
        doc.import_file(author.clone(), key.clone(), path_str, true, None)
            .await
            .unwrap();

        // export file
        let entry = doc
            .get_one(Query::author_key_exact(&author, key).into())
            .await
            .unwrap()
            .unwrap();
        let key = entry.key().to_vec();
        let out_root_str = out_root.to_string_lossy().into_owned();
        let path = crate::key_to_path(key, None, Some(out_root_str)).unwrap();
        doc.export_file(entry, path.clone(), None).await.unwrap();

        let got_bytes = tokio::fs::read(path).await.unwrap();
        assert_eq!(buf, got_bytes);
    }
}
