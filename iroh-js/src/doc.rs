use std::{path::PathBuf, str::FromStr};

use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use tracing::warn;

use crate::{AddrInfoOptions, AuthorId, DocTicket, Hash, Iroh, NodeAddr};

#[derive(Debug, Clone)]
#[napi(string_enum)]
pub enum CapabilityKind {
    /// A writable replica.
    Write,
    /// A readable replica.
    Read,
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
#[napi]
pub struct Docs {
    node: Iroh,
}

#[napi]
impl Iroh {
    /// Access to docs specific funtionaliy.
    #[napi(getter)]
    pub fn docs(&self) -> Docs {
        Docs { node: self.clone() }
    }
}

impl Docs {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.inner_client()
    }
}

#[napi]
impl Docs {
    /// Create a new doc.
    #[napi]
    pub async fn create(&self) -> Result<Doc> {
        let doc = self.client().docs().create().await?;

        Ok(Doc { inner: doc })
    }

    /// Join and sync with an already existing document.
    #[napi]
    pub async fn join(&self, ticket: &DocTicket) -> Result<Doc> {
        let ticket = ticket.try_into()?;
        let doc = self.client().docs().import(ticket).await?;
        Ok(Doc { inner: doc })
    }

    /// Join and sync with an already existing document and subscribe to events on that document.
    #[napi]
    pub async fn join_and_subscribe(
        &self,
        ticket: &DocTicket,
        cb: ThreadsafeFunction<LiveEvent, ()>,
    ) -> Result<Doc> {
        let ticket = ticket.try_into()?;
        let (doc, mut stream) = self.client().docs().import_and_subscribe(ticket).await?;

        tokio::spawn(async move {
            while let Some(event) = stream.next().await {
                let message: Result<LiveEvent> = event.map(Into::into).map_err(Into::into);
                if let Err(err) = cb.call_async(message).await {
                    warn!("cb error: {:?}", err);
                }
            }
        });

        Ok(Doc { inner: doc })
    }

    /// List all the docs we have access to on this node.
    #[napi]
    pub async fn list(&self) -> Result<Vec<NamespaceAndCapability>> {
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
    #[napi]
    pub async fn open(&self, id: String) -> Result<Option<Doc>> {
        let namespace_id = iroh::docs::NamespaceId::from_str(&id)?;
        let doc = self.client().docs().open(namespace_id).await?;

        Ok(doc.map(|d| Doc { inner: d }))
    }

    /// Delete a document from the local node.
    ///
    /// This is a destructive operation. Both the document secret key and all entries in the
    /// document will be permanently deleted from the node's storage. Content blobs will be deleted
    /// through garbage collection unless they are referenced from another document or tag.
    #[napi]
    pub async fn drop_doc(&self, doc_id: String) -> Result<()> {
        let doc_id = iroh::docs::NamespaceId::from_str(&doc_id)?;
        self.client().docs().drop_doc(doc_id).await?;
        Ok(())
    }
}

/// The namespace id and CapabilityKind (read/write) of the doc
#[derive(Debug)]
#[napi(object)]
pub struct NamespaceAndCapability {
    /// The namespace id of the doc
    pub namespace: String,
    /// The capability you have for the doc (read/write)
    pub capability: CapabilityKind,
}

/// A representation of a mutable, synchronizable key-value store.
#[derive(Clone)]
#[napi]
pub struct Doc {
    pub(crate) inner: iroh::client::Doc,
}

#[napi]
impl Doc {
    /// Get the document id of this doc.
    #[napi]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    /// Close the document.
    #[napi]
    pub async fn close_me(&self) -> Result<()> {
        self.inner.close().await?;
        Ok(())
    }

    /// Set the content of a key to a byte array.
    #[napi]
    pub async fn set_bytes(
        &self,
        author_id: &AuthorId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Hash> {
        let hash = self.inner.set_bytes(author_id.0, key, value).await?;
        Ok(hash.into())
    }

    /// Set an entries on the doc via its key, hash, and size.
    #[napi]
    pub async fn set_hash(
        &self,
        author_id: &AuthorId,
        key: Vec<u8>,
        hash: String,
        size: BigInt,
    ) -> Result<()> {
        self.inner
            .set_hash(
                author_id.0,
                key,
                hash.parse().map_err(anyhow::Error::from)?,
                size.get_u64().1,
            )
            .await?;
        Ok(())
    }

    /// Add an entry from an absolute file path
    #[napi]
    pub async fn import_file(
        &self,
        author: &AuthorId,
        key: Vec<u8>,
        path: String,
        in_place: bool,
        cb: Option<ThreadsafeFunction<DocImportProgress, ()>>,
    ) -> Result<()> {
        let mut stream = self
            .inner
            .import_file(author.0, Bytes::from(key), PathBuf::from(path), in_place)
            .await?;

        while let Some(event) = stream.next().await {
            if let Some(ref cb) = cb {
                let message = DocImportProgress::convert(event);
                cb.call_async(message).await?;
            }
        }
        Ok(())
    }

    /// Export an entry as a file to a given absolute path
    #[napi]
    pub async fn export_file(
        &self,
        entry: Entry,
        path: String,
        cb: Option<ThreadsafeFunction<DocExportProgress, ()>>,
    ) -> Result<()> {
        let mut stream = self
            .inner
            .export_file(
                entry.try_into()?,
                std::path::PathBuf::from(path),
                // TODO(b5) - plumb up the export mode, currently it's always copy
                iroh::blobs::store::ExportMode::Copy,
            )
            .await?;
        while let Some(event) = stream.next().await {
            if let Some(ref cb) = cb {
                let message = DocExportProgress::convert(event);
                cb.call_async(message).await?;
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
    #[napi]
    pub async fn delete(&self, author_id: &AuthorId, prefix: Vec<u8>) -> Result<u64> {
        let num_del = self.inner.del(author_id.0, prefix).await?;

        u64::try_from(num_del).map_err(|e| anyhow::Error::from(e).into())
    }

    /// Get an entry for a key and author.
    #[napi]
    pub async fn get_exact(
        &self,
        author: &AuthorId,
        key: Vec<u8>,
        include_empty: bool,
    ) -> Result<Option<Entry>> {
        let res = self
            .inner
            .get_exact(author.0, key, include_empty)
            .await
            .map(|e| e.map(|e| e.into()))?;
        Ok(res)
    }

    /// Get entries.
    ///
    /// Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi]
    pub async fn get_many(&self, query: &Query) -> Result<Vec<Entry>> {
        let entries = self
            .inner
            .get_many(query.0.clone())
            .await?
            .map_ok(|e| e.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(entries)
    }

    /// Get the latest entry for a key and author.
    #[napi]
    pub async fn get_one(&self, query: &Query) -> Result<Option<Entry>> {
        let res = self
            .inner
            .get_one(query.0.clone())
            .await
            .map(|e| e.map(|e| e.into()))?;
        Ok(res)
    }

    /// Share this document with peers over a ticket.
    #[napi]
    pub async fn share(&self, mode: ShareMode, addr_options: AddrInfoOptions) -> Result<DocTicket> {
        let res = self
            .inner
            .share(mode.into(), addr_options.into())
            .await
            .map(|ticket| ticket.into())?;
        Ok(res)
    }

    /// Start to sync this document with a list of peers.
    #[napi]
    pub async fn start_sync(&self, peers: Vec<NodeAddr>) -> Result<()> {
        let peers = peers
            .into_iter()
            .map(|p| p.try_into())
            .collect::<anyhow::Result<Vec<_>>>()?;

        self.inner.start_sync(peers).await?;
        Ok(())
    }

    /// Stop the live sync for this document.
    #[napi]
    pub async fn leave(&self) -> Result<()> {
        self.inner.leave().await?;
        Ok(())
    }

    /// Subscribe to events for this document.
    #[napi]
    pub async fn subscribe(&self, cb: ThreadsafeFunction<LiveEvent, ()>) -> Result<()> {
        let client = self.inner.clone();
        tokio::task::spawn(async move {
            let mut sub = client.subscribe().await.unwrap();
            while let Some(event) = sub.next().await {
                let message: Result<LiveEvent> = event.map(Into::into).map_err(Into::into);
                if let Err(err) = cb.call_async(message).await {
                    warn!("cb error: {:?}", err);
                }
            }
        });

        Ok(())
    }

    /// Get status info for this document
    #[napi]
    pub async fn status(&self) -> Result<OpenState> {
        let res = self.inner.status().await.map(|o| o.into())?;
        Ok(res)
    }

    /// Set the download policy for this document
    #[napi]
    pub async fn set_download_policy(&self, policy: &DownloadPolicy) -> Result<()> {
        self.inner.set_download_policy(policy.into()).await?;
        Ok(())
    }

    /// Get the download policy for this document
    #[napi]
    pub async fn get_download_policy(&self) -> Result<DownloadPolicy> {
        let res = self
            .inner
            .get_download_policy()
            .await
            .map(|policy| policy.into())?;
        Ok(res)
    }

    /// Get sync peers for this document
    #[napi]
    pub async fn get_sync_peers(&self) -> Result<Option<Vec<Vec<u8>>>> {
        let list = self.inner.get_sync_peers().await?;
        let list = list.map(|l| l.into_iter().map(|p| p.to_vec()).collect());
        Ok(list)
    }
}

/// Download policy to decide which content blobs shall be downloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct DownloadPolicy {
    /// Do not download any key unless it matches one of the filters.
    nothing_except: Option<Vec<FilterKind>>,
    /// Download every key unless it matches one of the filters.
    everything_except: Option<Vec<FilterKind>>,
}

#[napi]
impl DownloadPolicy {
    /// Download everything
    #[napi(factory)]
    pub fn everything() -> Self {
        DownloadPolicy {
            everything_except: Some(Vec::new()),
            nothing_except: None,
        }
    }

    /// Download nothing
    #[napi(factory)]
    pub fn nothing() -> Self {
        DownloadPolicy {
            everything_except: None,
            nothing_except: Some(Vec::new()),
        }
    }

    /// Download nothing except keys that match the given filters
    #[napi(factory)]
    pub fn nothing_except(filters: Vec<&FilterKind>) -> Self {
        DownloadPolicy {
            everything_except: None,
            nothing_except: Some(filters.into_iter().cloned().collect()),
        }
    }

    /// Download everything except keys that match the given filters
    #[napi(factory)]
    pub fn everything_except(filters: Vec<&FilterKind>) -> Self {
        DownloadPolicy {
            everything_except: Some(filters.into_iter().cloned().collect()),
            nothing_except: None,
        }
    }
}

impl From<iroh::docs::store::DownloadPolicy> for DownloadPolicy {
    fn from(value: iroh::docs::store::DownloadPolicy) -> Self {
        match value {
            iroh::docs::store::DownloadPolicy::NothingExcept(filters) => DownloadPolicy {
                nothing_except: Some(filters.into_iter().map(|f| f.into()).collect()),
                everything_except: None,
            },
            iroh::docs::store::DownloadPolicy::EverythingExcept(filters) => DownloadPolicy {
                everything_except: Some(filters.into_iter().map(|f| f.into()).collect()),
                nothing_except: None,
            },
        }
    }
}

impl From<&DownloadPolicy> for iroh::docs::store::DownloadPolicy {
    fn from(value: &DownloadPolicy) -> Self {
        if let Some(ref filters) = value.nothing_except {
            return iroh::docs::store::DownloadPolicy::NothingExcept(
                filters.iter().map(|f| f.0.clone()).collect(),
            );
        }

        if let Some(ref filters) = value.everything_except {
            return iroh::docs::store::DownloadPolicy::EverythingExcept(
                filters.iter().map(|f| f.0.clone()).collect(),
            );
        }

        panic!("invalid internal state");
    }
}

/// Filter strategy used in download policies.
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct FilterKind(pub(crate) iroh::docs::store::FilterKind);

#[napi]
impl FilterKind {
    /// Verifies whether this filter matches a given key
    #[napi]
    pub fn matches(&self, key: Vec<u8>) -> bool {
        self.0.matches(key)
    }

    /// Returns a FilterKind that matches if the contained bytes are a prefix of the key.
    #[napi(factory)]
    pub fn prefix(prefix: Vec<u8>) -> FilterKind {
        FilterKind(iroh::docs::store::FilterKind::Prefix(Bytes::from(prefix)))
    }

    /// Returns a FilterKind that matches if the contained bytes and the key are the same.
    #[napi(factory)]
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
#[derive(Debug, Clone)]
#[napi(object)]
pub struct OpenState {
    /// Whether to accept sync requests for this replica.
    pub sync: bool,
    /// How many event subscriptions are open
    pub subscribers: BigInt,
    /// By how many handles the replica is currently held open
    pub handles: BigInt,
}

impl From<iroh::docs::actor::OpenState> for OpenState {
    fn from(value: iroh::docs::actor::OpenState) -> Self {
        OpenState {
            sync: value.sync,
            subscribers: (value.subscribers as u64).into(),
            handles: (value.handles as u64).into(),
        }
    }
}

/// Intended capability for document share tickets
#[derive(Debug)]
#[napi(string_enum)]
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
#[derive(Debug, Clone)]
#[napi(object)]
pub struct Entry {
    /// The namespace this entry belongs to
    pub namespace: String,
    /// The author of the entry
    pub author: String,
    /// The key of the entry.
    pub key: Vec<u8>,
    /// Length of the data referenced by `hash`.
    pub len: BigInt,
    /// Hash of the content data.
    pub hash: String,
    /// Record creation timestamp. Counted as micros since the Unix epoch.
    pub timestamp: BigInt,
}

impl From<iroh::client::docs::Entry> for Entry {
    fn from(e: iroh::client::docs::Entry) -> Self {
        Self {
            namespace: e.id().namespace().to_string(),
            author: e.author().to_string(),
            key: e.key().to_vec(),
            len: e.content_len().into(),
            hash: e.content_hash().to_string(),
            timestamp: e.timestamp().into(),
        }
    }
}

impl TryFrom<Entry> for iroh::client::docs::Entry {
    type Error = anyhow::Error;
    fn try_from(value: Entry) -> std::prelude::v1::Result<Self, Self::Error> {
        let author: iroh::docs::AuthorId = value.author.parse()?;
        let namespace: iroh::docs::NamespaceId = value.namespace.parse()?;
        let id = iroh::docs::RecordIdentifier::new(namespace, author, value.key);
        let hash: iroh::blobs::Hash = value.hash.parse()?;
        let len = value.len.get_u64().1;
        let timestamp = value.timestamp.get_u64().1;
        let record = iroh::docs::Record::new(hash, len, timestamp);
        let entry = iroh::docs::Entry::new(id, record);
        Ok(entry.into())
    }
}

/// Fields by which the query can be sorted
#[derive(Debug, Default, Clone)]
#[napi(string_enum)]
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
#[derive(Debug, Default, Clone)]
#[napi(string_enum)]
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
#[derive(Clone, Debug)]
#[napi]
pub struct Query(pub(crate) iroh::docs::store::Query);

/// Options for sorting and pagination for using [`Query`]s.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct QueryOptions {
    /// Sort by author or key first.
    ///
    /// Default is [`SortBy::AuthorKey`], so sorting first by author and then by key.
    pub sort_by: Option<SortBy>,
    /// Direction by which to sort the entries
    ///
    /// Default is [`SortDirection::Asc`]
    pub direction: Option<SortDirection>,
    /// Offset
    pub offset: Option<BigInt>,
    /// Limit to limit the pagination.
    ///
    /// When the limit is 0, the limit does not exist.
    pub limit: Option<BigInt>,
}

impl QueryOptions {
    fn offset(&self) -> Option<u64> {
        self.offset
            .as_ref()
            .map(|o| o.get_u64().1)
            .and_then(|o| if o == 0 { None } else { Some(o) })
    }

    fn limit(&self) -> Option<u64> {
        self.limit
            .as_ref()
            .map(|o| o.get_u64().1)
            .and_then(|o| if o == 0 { None } else { Some(o) })
    }
}

#[napi]
impl Query {
    /// Query all records.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(factory)]
    pub fn all(opts: Option<QueryOptions>) -> Self {
        let builder = iroh::docs::store::Query::all();
        let builder = apply_opts_with_sort(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Query only the latest entry for each key, omitting older entries if the entry was written
    /// to by multiple authors.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(factory)]
    pub fn single_latest_per_key(opts: Option<QueryOptions>) -> Self {
        let builder = iroh::docs::store::Query::single_latest_per_key();
        let builder = apply_opts(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Query exactly the key, but only the latest entry for it, omitting older entries if the entry was written
    /// to by multiple authors.
    #[napi(factory)]
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
    #[napi(factory)]
    pub fn single_latest_per_key_prefix(prefix: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let builder = iroh::docs::store::Query::single_latest_per_key().key_prefix(prefix);
        let builder = apply_opts(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Query all entries for by a single author.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(factory)]
    pub fn author(author: &AuthorId, opts: Option<QueryOptions>) -> Self {
        let builder = iroh::docs::store::Query::author(author.0);
        let builder = apply_opts_with_sort(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Query all entries that have an exact key.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(factory)]
    pub fn key_exact(key: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let builder = iroh::docs::store::Query::key_exact(key);
        let builder = apply_opts_with_sort(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Create a Query for a single key and author.
    #[napi(factory)]
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
    #[napi(factory)]
    pub fn key_prefix(prefix: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let builder = iroh::docs::store::Query::key_prefix(prefix);
        let builder = apply_opts_with_sort(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Create a query for all entries of a single author with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(factory)]
    pub fn author_key_prefix(
        author: &AuthorId,
        prefix: Vec<u8>,
        opts: Option<QueryOptions>,
    ) -> Self {
        let builder = iroh::docs::store::Query::author(author.0).key_prefix(prefix);
        let builder = apply_opts_with_sort(builder, opts.as_ref());
        Query(builder.build())
    }

    /// Get the limit for this query (max. number of entries to emit).
    #[napi]
    pub fn limit(&self) -> Option<BigInt> {
        self.0.limit().map(Into::into)
    }

    /// Get the offset for this query (number of entries to skip at the beginning).
    #[napi]
    pub fn offset(&self) -> BigInt {
        self.0.offset().into()
    }
}

fn apply_opts_with_sort(
    mut builder: iroh::docs::store::QueryBuilder<iroh::docs::store::FlatQuery>,
    opts: Option<&QueryOptions>,
) -> iroh::docs::store::QueryBuilder<iroh::docs::store::FlatQuery> {
    builder = apply_opts(builder, opts);
    if let Some(opts) = opts {
        if let Some(ref sort_by) = opts.sort_by {
            let direction = opts.direction.clone().unwrap_or_default();
            builder = builder.sort_by(sort_by.clone().into(), direction.into());
        }
    }
    builder
}

fn apply_opts<K>(
    mut builder: iroh::docs::store::QueryBuilder<K>,
    opts: Option<&QueryOptions>,
) -> iroh::docs::store::QueryBuilder<K> {
    if let Some(opts) = opts {
        if let Some(offset) = opts.offset() {
            builder = builder.offset(offset);
        }
        if let Some(limit) = opts.limit() {
            builder = builder.limit(limit);
        }
    }
    builder
}

/// Events informing about actions of the live sync progress
#[derive(Debug, Default)]
#[napi(object)]
pub struct LiveEvent {
    /// A local insertion.
    pub insert_local: Option<LiveEventInsertLocal>,
    /// Received a remote insert.
    pub insert_remote: Option<LiveEventInsertRemote>,
    /// The content of an entry was downloaded and is now available at the local node
    pub content_ready: Option<LiveEventContentReady>,
    /// We have a new neighbor in the swarm.
    pub neighbor_up: Option<LiveEventNeighborUp>,
    /// We lost a neighbor in the swarm.
    pub neighbor_down: Option<LiveEventNeighborDown>,
    /// A set-reconciliation sync finished.
    pub sync_finished: Option<SyncEvent>,
    /// All pending content is now ready.
    ///
    /// This event signals that all queued content downloads from the last sync run have either
    /// completed or failed.
    ///
    /// It will only be emitted after a [`Self::SyncFinished`] event, never before.
    ///
    /// Receiving this event does not guarantee that all content in the document is available. If
    /// blobs failed to download, this event will still be emitted after all operations completed.
    pub pending_content_ready: bool,
}

#[derive(Debug)]
#[napi(object)]
pub struct LiveEventInsertLocal {
    /// The inserted entry.
    pub entry: Entry,
}

#[derive(Debug)]
#[napi(object)]
pub struct LiveEventInsertRemote {
    /// The peer that sent us the entry.
    pub from: String,
    /// The inserted entry.
    pub entry: Entry,
    /// If the content is available at the local node
    pub content_status: ContentStatus,
}

#[derive(Debug)]
#[napi(object)]
pub struct LiveEventContentReady {
    /// The content hash of the newly available entry content
    pub hash: String,
}

#[derive(Debug)]
#[napi(object)]
pub struct LiveEventNeighborUp {
    /// Public key of the neighbor
    pub neighbor: String,
}

#[derive(Debug)]
#[napi(object)]
pub struct LiveEventNeighborDown {
    /// Public key of the neighbor
    pub neighbor: String,
}

impl From<iroh::client::docs::LiveEvent> for LiveEvent {
    fn from(value: iroh::client::docs::LiveEvent) -> Self {
        match value {
            iroh::client::docs::LiveEvent::InsertLocal { entry } => LiveEvent {
                insert_local: Some(LiveEventInsertLocal {
                    entry: entry.into(),
                }),
                ..Default::default()
            },
            iroh::client::docs::LiveEvent::InsertRemote {
                from,
                entry,
                content_status,
            } => LiveEvent {
                insert_remote: Some(LiveEventInsertRemote {
                    from: from.to_string(),
                    entry: entry.into(),
                    content_status: content_status.into(),
                }),
                ..Default::default()
            },
            iroh::client::docs::LiveEvent::ContentReady { hash } => LiveEvent {
                content_ready: Some(LiveEventContentReady {
                    hash: hash.to_string(),
                }),
                ..Default::default()
            },
            iroh::client::docs::LiveEvent::NeighborUp(key) => LiveEvent {
                neighbor_up: Some(LiveEventNeighborUp {
                    neighbor: key.to_string(),
                }),
                ..Default::default()
            },
            iroh::client::docs::LiveEvent::NeighborDown(key) => LiveEvent {
                neighbor_down: Some(LiveEventNeighborDown {
                    neighbor: key.to_string(),
                }),
                ..Default::default()
            },
            iroh::client::docs::LiveEvent::SyncFinished(e) => LiveEvent {
                sync_finished: Some(e.into()),
                ..Default::default()
            },
            iroh::client::docs::LiveEvent::PendingContentReady => LiveEvent {
                pending_content_ready: true,
                ..Default::default()
            },
        }
    }
}

/// Outcome of a sync operation
#[derive(Debug, Clone)]
#[napi(object)]
pub struct SyncEvent {
    /// Peer we synced with
    pub peer: String,
    /// Origin of the sync exchange
    pub origin: Origin,
    /// Timestamp when the sync finished
    pub finished: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the sync started
    pub started: chrono::DateTime<chrono::Utc>,
    /// Result of the sync operation. `None` if successfull.
    pub result: Option<String>,
}

impl From<iroh::client::docs::SyncEvent> for SyncEvent {
    fn from(value: iroh::client::docs::SyncEvent) -> Self {
        SyncEvent {
            peer: value.peer.to_string(),
            origin: value.origin.into(),
            finished: value.finished.into(),
            started: value.started.into(),
            result: match value.result {
                Ok(_) => None,
                Err(err) => Some(err),
            },
        }
    }
}

/// Why we started a sync request
#[derive(Debug, Eq, PartialEq)]
#[napi(string_enum)]
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
#[derive(Debug, Clone)]
#[napi(string_enum)]
pub enum Origin {
    /// Direct join request via API
    ConnectDirectJoin,
    /// Peer showed up as new neighbor in the gossip swarm
    ConnectNewNeighbor,
    /// We synced after receiving a sync report that indicated news for us
    ConnectSyncReport,
    /// We received a sync report while a sync was running, so run again afterwars
    ConnectResync,
    /// A peer connected to us and we accepted the exchange
    Accept,
}

impl From<iroh::client::docs::Origin> for Origin {
    fn from(value: iroh::client::docs::Origin) -> Self {
        match value {
            iroh::client::docs::Origin::Connect(reason) => match reason {
                iroh::client::docs::SyncReason::DirectJoin => Self::ConnectDirectJoin,
                iroh::client::docs::SyncReason::NewNeighbor => Self::ConnectNewNeighbor,
                iroh::client::docs::SyncReason::SyncReport => Self::ConnectSyncReport,
                iroh::client::docs::SyncReason::Resync => Self::ConnectResync,
            },
            iroh::client::docs::Origin::Accept => Self::Accept,
        }
    }
}

/// Outcome of an InsertRemove event.
#[derive(Debug)]
#[napi(object)]
pub struct InsertRemoteEvent {
    /// The peer that sent us the entry.
    pub from: String,
    /// The inserted entry.
    pub entry: Entry,
    /// If the content is available at the local node
    pub content_status: ContentStatus,
}

/// Whether the content status is available on a node.
#[derive(Debug)]
#[napi(string_enum)]
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

/// The type of `DocImportProgress` event
#[derive(Debug, PartialEq, Eq)]
#[napi(string_enum)]
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
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocImportProgressFound {
    /// A new unique id for this entry.
    pub id: BigInt,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: BigInt,
}

/// A DocImportProgress event indicating we've made progress ingesting item `id`.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocImportProgressProgress {
    /// The unique id of the entry.
    pub id: BigInt,
    /// The offset of the progress, in bytes.
    pub offset: BigInt,
}

/// A DocImportProgress event indicating we are finished adding `id` to the data store and the hash is `hash`.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocImportProgressIngestDone {
    /// The unique id of the entry.
    pub id: BigInt,
    /// The hash of the entry.
    pub hash: String,
}

/// A DocImportProgress event indicating we are done setting the entry to the doc
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocImportProgressAllDone {
    /// The key of the entry
    pub key: Vec<u8>,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Default)]
#[napi(object)]
pub struct DocImportProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    pub found: Option<DocImportProgressFound>,
    /// We got progress ingesting item `id`.
    pub progress: Option<DocImportProgressProgress>,
    /// We are done ingesting `id`, and the hash is `hash`.
    pub ingest_done: Option<DocImportProgressIngestDone>,
    /// We are done with the whole operation.
    pub all_done: Option<DocImportProgressAllDone>,
}

impl DocImportProgress {
    fn convert(value: anyhow::Result<iroh::client::docs::ImportProgress>) -> Result<Self> {
        match value {
            Ok(iroh::client::docs::ImportProgress::Found { id, name, size }) => {
                Ok(DocImportProgress {
                    found: Some(DocImportProgressFound {
                        id: id.into(),
                        name,
                        size: size.into(),
                    }),
                    ..Default::default()
                })
            }
            Ok(iroh::client::docs::ImportProgress::Progress { id, offset }) => {
                Ok(DocImportProgress {
                    progress: Some(DocImportProgressProgress {
                        id: id.into(),
                        offset: offset.into(),
                    }),
                    ..Default::default()
                })
            }
            Ok(iroh::client::docs::ImportProgress::IngestDone { id, hash }) => {
                Ok(DocImportProgress {
                    ingest_done: Some(DocImportProgressIngestDone {
                        id: id.into(),
                        hash: hash.to_string(),
                    }),
                    ..Default::default()
                })
            }
            Ok(iroh::client::docs::ImportProgress::AllDone { key }) => Ok(DocImportProgress {
                all_done: Some(DocImportProgressAllDone { key: key.into() }),
                ..Default::default()
            }),
            Ok(iroh::client::docs::ImportProgress::Abort(err)) => {
                Err(anyhow::Error::from(err).into())
            }
            Err(err) => Err(err.into()),
        }
    }
}

/// A DocExportProgress event indicating a file was found with name `name`, from now on referred to via `id`
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocExportProgressFound {
    /// A new unique id for this entry.
    pub id: BigInt,
    /// The hash of the entry.
    pub hash: String,
    /// The size of the entry in bytes.
    pub size: BigInt,
    /// The path where we are writing the entry
    pub outpath: String,
}

/// A DocExportProgress event indicating we've made progress exporting item `id`.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocExportProgressProgress {
    /// The unique id of the entry.
    pub id: BigInt,
    /// The offset of the progress, in bytes.
    pub offset: BigInt,
}

/// A DocExportProgress event indicating a single blob wit `id` is done
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocExportProgressDone {
    /// The unique id of the entry.
    pub id: BigInt,
}

/// A DocExportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DocExportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Default)]
#[napi(object)]
pub struct DocExportProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    pub found: Option<DocExportProgressFound>,
    /// We got progress ingesting item `id`.
    pub progress: Option<DocExportProgressProgress>,
    /// We finished exporting a blob
    pub done: Option<DocExportProgressDone>,
    /// We are done with the whole operation.
    pub all_done: bool,
}

impl DocExportProgress {
    fn convert(value: anyhow::Result<iroh::blobs::export::ExportProgress>) -> Result<Self> {
        match value {
            Ok(value) => match value {
                iroh::blobs::export::ExportProgress::Found {
                    id,
                    hash,
                    size,
                    outpath,
                    ..
                } => Ok(DocExportProgress {
                    found: Some(DocExportProgressFound {
                        id: id.into(),
                        hash: hash.to_string(),
                        size: size.value().into(),
                        outpath: outpath.to_string_lossy().to_string(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::export::ExportProgress::Progress { id, offset } => {
                    Ok(DocExportProgress {
                        progress: Some(DocExportProgressProgress {
                            id: id.into(),
                            offset: offset.into(),
                        }),
                        ..Default::default()
                    })
                }
                iroh::blobs::export::ExportProgress::Done { id } => Ok(DocExportProgress {
                    done: Some(DocExportProgressDone { id: id.into() }),
                    ..Default::default()
                }),
                iroh::blobs::export::ExportProgress::AllDone => Ok(DocExportProgress {
                    all_done: true,
                    ..Default::default()
                }),
                iroh::blobs::export::ExportProgress::Abort(err) => {
                    Err(anyhow::Error::from(err).into())
                }
            },
            Err(err) => Err(err.into()),
        }
    }
}
