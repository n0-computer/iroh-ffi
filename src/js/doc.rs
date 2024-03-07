use std::str::FromStr;

use futures::{StreamExt, TryStreamExt};
use iroh::{
    client::Doc as ClientDoc,
    rpc_protocol::{ProviderRequest, ProviderResponse},
};
use napi::bindgen_prelude::{BigInt, Buffer, Generator};
use napi_derive::napi;
use quic_rpc::transport::flume::FlumeConnection;

use crate::{
    AuthorId, DownloadPolicy, Entry, Hash, IrohNode, NamespaceAndCapability, NodeAddr, OpenState,
    Query, QueryOptions, ShareMode, SortBy, SortDirection,
};

use super::u64_from_bigint;

#[napi]
impl IrohNode {
    #[napi(js_name = "docCreate")]
    pub async fn doc_create_js(&self) -> napi::Result<JsDoc> {
        Ok(JsDoc::new(self).await)
    }

    /// Join and sync with an already existing document.
    #[napi(js_name = "docJoin")]
    pub async fn doc_join_js(&self, ticket: String) -> napi::Result<JsDoc> {
        let ticket = iroh::ticket::DocTicket::from_str(&ticket).map_err(anyhow::Error::from)?;
        let doc = self.sync_client.docs.import(ticket).await?;

        Ok(JsDoc { inner: doc })
    }

    /// List all the docs we have access to on this node.
    #[napi(js_name = "docList")]
    pub async fn doc_list_js(&self) -> napi::Result<Vec<NamespaceAndCapability>> {
        let docs = self
            .sync_client
            .docs
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
    #[napi(js_name = "docOpen")]
    pub async fn doc_open_js(&self, id: String) -> napi::Result<Option<JsDoc>> {
        let namespace_id = iroh::sync::NamespaceId::from_str(&id)?;
        let doc = self.sync_client.docs.open(namespace_id).await?;

        Ok(doc.map(|d| JsDoc { inner: d }))
    }

    /// Delete a document from the local node.
    ///
    /// This is a destructive operation. Both the document secret key and all entries in the
    /// document will be permanently deleted from the node's storage. Content blobs will be delted
    /// through garbage collection unless they are referenced from another document or tag.
    #[napi(js_name = "docDrop")]
    pub async fn doc_drop_js(&self, doc_id: String) -> napi::Result<()> {
        let doc_id = iroh::sync::NamespaceId::from_str(&doc_id)?;
        self.sync_client.docs.drop_doc(doc_id).await?;
        Ok(())
    }
}

/// A representation of a mutable, synchronizable key-value store.
#[napi(js_name = "Doc")]
#[derive(Clone)]
pub struct JsDoc {
    pub(crate) inner: ClientDoc<FlumeConnection<ProviderResponse, ProviderRequest>>,
}

#[napi]
impl JsDoc {
    #[napi(constructor)]
    pub async fn new(node: &IrohNode) -> JsDoc {
        let doc = node.sync_client.docs.create().await.unwrap();

        JsDoc { inner: doc }
    }

    /// Get the document id of this doc.
    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    /// Close the document.
    #[napi]
    pub async fn close(&self) -> Result<(), napi::Error> {
        self.inner.close().await?;
        Ok(())
    }

    /// Set the content of a key to a byte array.
    #[napi]
    pub async fn set_bytes(
        &self,
        author_id: &AuthorId,
        key: Buffer,
        value: Buffer,
    ) -> Result<Hash, napi::Error> {
        let key: Vec<_> = key.into();
        let value: Vec<_> = value.into();
        let hash = self.inner.set_bytes(author_id.0, key, value).await?;

        Ok(Hash(hash))
    }

    /// Set an entries on the doc via its key, hash, and size.
    #[napi]
    pub async fn set_hash(
        &self,
        author_id: &AuthorId,
        key: Buffer,
        hash: &Hash,
        size: napi::bindgen_prelude::BigInt,
    ) -> Result<(), napi::Error> {
        let key: Vec<_> = key.into();
        let size = super::u64_from_bigint(size)?;
        self.inner.set_hash(author_id.0, key, hash.0, size).await?;

        Ok(())
    }

    /// Add an entry from an absolute file path
    #[napi]
    pub async fn import_file(
        &self,
        author: &AuthorId,
        key: Buffer,
        path: String,
        in_place: bool,
    ) -> Result<JsDocImportProgress, napi::Error> {
        let key: Vec<_> = key.into();
        let mut stream = self
            .inner
            .import_file(
                author.0,
                bytes::Bytes::from(key),
                std::path::PathBuf::from(path),
                in_place,
            )
            .await?;

        // arbitrary channel size
        let (send, recv) = flume::bounded(64);
        let handle = tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                send.send(res).expect("receiver dropped");
            }
        });
        Ok(JsDocImportProgress { recv, handle })
    }

    /// Export an entry as a file to a given absolute path
    #[napi]
    pub async fn export_file(
        &self,
        entry: &Entry,
        path: String,
    ) -> Result<JsDocExportProgress, napi::Error> {
        let mut stream = self
            .inner
            .export_file(entry.0.clone(), std::path::PathBuf::from(path))
            .await?;

        // arbitrary channel size
        let (send, recv) = flume::bounded(64);
        let handle = tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                send.send(res).expect("receiver dropped");
            }
        });
        Ok(JsDocExportProgress { recv, handle })
    }

    /// Delete entries that match the given `author` and key `prefix`.
    ///
    /// This inserts an empty entry with the key set to `prefix`, effectively clearing all other
    /// entries whose key starts with or is equal to the given `prefix`.
    ///
    /// Returns the number of entries deleted.
    #[napi]
    pub async fn del(&self, author_id: &AuthorId, prefix: Buffer) -> Result<u64, napi::Error> {
        let prefix: Vec<_> = prefix.into();
        let num_del = self.inner.del(author_id.0, prefix).await?;
        Ok(num_del as u64)
    }

    /// Get an entry for a key and author.
    #[napi]
    pub async fn get_exact(
        &self,
        author: &AuthorId,
        key: Buffer,
        include_empty: bool,
    ) -> Result<Option<Entry>, napi::Error> {
        let key: Vec<_> = key.into();
        let e = self.inner.get_exact(author.0, key, include_empty).await?;
        Ok(e.map(Into::into))
    }

    /// Get entries.
    ///
    /// Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi]
    pub async fn get_many(&self, query: &Query) -> Result<Vec<Entry>, napi::Error> {
        let entries = self
            .inner
            .get_many(query.0.clone())
            .await?
            .map_ok(Entry)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(entries)
    }

    /// Get the latest entry for a key and author.
    #[napi]
    pub async fn get_one(&self, query: &Query) -> Result<Option<Entry>, napi::Error> {
        let e = self.inner.get_one((*query).clone().0).await?;
        Ok(e.map(Into::into))
    }

    /// Share this document with peers over a ticket.
    #[napi]
    pub async fn share(&self, mode: ShareMode) -> Result<String, napi::Error> {
        let ticket = self.inner.share(mode.into()).await?;
        Ok(ticket.to_string())
    }

    /// Start to sync this document with this peer.
    #[napi]
    pub async fn start_sync(&self, peer: &NodeAddr) -> Result<(), napi::Error> {
        let peer = (*peer).clone().try_into().map_err(anyhow::Error::from)?;
        let peers = vec![peer];

        self.inner.start_sync(peers).await?;
        Ok(())
    }

    /// Stop the live sync for this document.
    #[napi]
    pub async fn leave(&self) -> Result<(), napi::Error> {
        self.inner.leave().await?;
        Ok(())
    }

    /// Subscribe to events for this document.
    #[napi]
    pub async fn subscribe(&self) -> Result<DocSubscriber, napi::Error> {
        let mut sub = self.inner.subscribe().await.unwrap();
        // arbitrary channel size
        let (send, recv) = flume::bounded(64);
        let handle = tokio::spawn(async move {
            while let Some(res) = sub.next().await {
                send.send(res).expect("receiver dropped");
            }
        });
        Ok(DocSubscriber { recv, handle })
    }

    /// Get status info for this document
    #[napi]
    pub async fn status(&self) -> Result<serde_json::Value, napi::Error> {
        let state = self.inner.status().await.map(OpenState::from)?;
        Ok(serde_json::to_value(state).unwrap())
    }

    // TODO:
    // /// Set the download policy for this document
    // #[napi]
    // pub async fn set_download_policy(&self, policy: &DownloadPolicy) -> Result<(), napi::Error> {
    //     self.inner
    //         .set_download_policy((*policy).clone().into())
    //         .await?;
    //     Ok(())
    // }

    /// Get the download policy for this document
    #[napi]
    pub async fn get_download_policy(&self) -> Result<serde_json::Value, napi::Error> {
        let policy = self
            .inner
            .get_download_policy()
            .await
            .map(DownloadPolicy::from)?;
        Ok(serde_json::to_value(policy).unwrap())
    }
}

#[napi(iterator)]
pub struct DocSubscriber {
    recv: flume::Receiver<anyhow::Result<iroh::client::LiveEvent>>,
    handle: tokio::task::JoinHandle<()>,
}

#[napi]
impl Generator for DocSubscriber {
    type Yield = serde_json::Value;
    type Next = serde_json::Value;
    type Return = ();

    fn next(&mut self, _value: Option<Self::Next>) -> Option<Self::Yield> {
        self.recv
            .recv()
            .ok()
            .and_then(|event| event.ok())
            .and_then(|event| serde_json::to_value(event).ok())
    }

    fn complete(&mut self, _value: Option<Self::Return>) -> Option<Self::Yield> {
        self.handle.abort();
        None
    }

    fn catch(
        &mut self,
        _env: napi::Env,
        value: napi::JsUnknown,
    ) -> Result<Option<Self::Yield>, napi::JsUnknown> {
        self.handle.abort();
        Err(value)
    }
}

#[napi(iterator)]
pub struct JsDocImportProgress {
    recv: flume::Receiver<anyhow::Result<iroh::rpc_protocol::DocImportProgress>>,
    handle: tokio::task::JoinHandle<()>,
}

#[napi]
impl Generator for JsDocImportProgress {
    type Yield = serde_json::Value;
    type Next = serde_json::Value;
    type Return = serde_json::Value;

    fn next(&mut self, _value: Option<Self::Next>) -> Option<Self::Yield> {
        self.recv
            .recv()
            .ok()
            .and_then(|event| event.ok())
            .and_then(|event| serde_json::to_value(event).ok())
    }

    fn complete(&mut self, _value: Option<Self::Return>) -> Option<Self::Yield> {
        let mut res = None;
        while let Ok(Ok(progress)) = self.recv.recv() {
            match progress {
                iroh::rpc_protocol::DocImportProgress::AllDone { .. }
                | iroh::rpc_protocol::DocImportProgress::Abort(_) => {
                    res = serde_json::to_value(progress).ok();
                    break;
                }
                _ => {}
            }
        }
        self.handle.abort();
        res
    }

    fn catch(
        &mut self,
        _env: napi::Env,
        value: napi::JsUnknown,
    ) -> Result<Option<Self::Yield>, napi::JsUnknown> {
        self.handle.abort();
        Err(value)
    }
}

#[napi(iterator)]
pub struct JsDocExportProgress {
    recv: flume::Receiver<anyhow::Result<iroh::rpc_protocol::DocExportProgress>>,
    handle: tokio::task::JoinHandle<()>,
}

#[napi]
impl Generator for JsDocExportProgress {
    type Yield = serde_json::Value;
    type Next = serde_json::Value;
    type Return = serde_json::Value;

    fn next(&mut self, _value: Option<Self::Next>) -> Option<Self::Yield> {
        self.recv
            .recv()
            .ok()
            .and_then(|event| event.ok())
            .and_then(|event| serde_json::to_value(event).ok())
    }

    fn complete(&mut self, _value: Option<Self::Return>) -> Option<Self::Yield> {
        let mut res = None;
        while let Ok(Ok(progress)) = self.recv.recv() {
            match progress {
                iroh::rpc_protocol::DocExportProgress::AllDone { .. }
                | iroh::rpc_protocol::DocExportProgress::Abort(_) => {
                    res = serde_json::to_value(progress).ok();
                    break;
                }
                _ => {}
            }
        }
        self.handle.abort();
        res
    }

    fn catch(
        &mut self,
        _env: napi::Env,
        value: napi::JsUnknown,
    ) -> Result<Option<Self::Yield>, napi::JsUnknown> {
        self.handle.abort();
        Err(value)
    }
}

#[napi]
impl Entry {
    /// Read all content of an [`Entry`] into a buffer.
    /// This allocates a buffer for the full entry. Use only if you know that the entry you're
    /// reading is small. If not sure, use [`Self::content_len`] and check the size with
    /// before calling [`Self::content_bytes`].
    #[napi(js_name = "contentBytes")]
    pub async fn content_bytes_js(&self, doc: &JsDoc) -> Result<Buffer, napi::Error> {
        let content = self.0.content_bytes(&doc.inner).await.map(|c| c.to_vec())?;
        Ok(content.into())
    }

    /// Get the content_length of this entry.
    #[napi(js_name = "contentLen")]
    pub fn content_len_js(&self) -> u64 {
        self.0.content_len()
    }
}

#[napi(object, js_name = "QueryOptions")]
#[derive(Clone, Debug)]
pub struct JsQueryOptions {
    /// Sort by author or key first.
    ///
    /// Default is [`SortBy::AuthorKey`], so sorting first by author and then by key.
    pub sort_by: SortBy,
    /// Direction by which to sort the entries
    ///
    /// Default is [`SortDirection::Asc`]
    pub direction: SortDirection,
    /// Offset
    pub offset: BigInt,
    /// Limit to limit the pagination.
    ///
    /// When the limit is 0, the limit does not exist.
    pub limit: BigInt,
}

impl TryFrom<JsQueryOptions> for QueryOptions {
    type Error = napi::Error;

    fn try_from(value: JsQueryOptions) -> Result<Self, Self::Error> {
        let offset = u64_from_bigint(value.offset)?;
        let limit = u64_from_bigint(value.limit)?;
        Ok(Self {
            sort_by: value.sort_by,
            direction: value.direction,
            offset,
            limit,
        })
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
    #[napi(js_name = "all")]
    pub fn all_js(opts: Option<JsQueryOptions>) -> napi::Result<Self> {
        let opts = opts.map(|o| o.try_into()).transpose()?;
        Ok(Query::all(opts))
    }

    /// Query only the latest entry for each key, omitting older entries if the entry was written
    /// to by multiple authors.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(js_name = "singleLatestPerKey")]
    pub fn single_latest_per_key_js(opts: Option<JsQueryOptions>) -> napi::Result<Self> {
        let opts = opts.map(|o| o.try_into()).transpose()?;
        Ok(Query::single_latest_per_key(opts))
    }

    /// Query all entries for by a single author.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(js_name = "author")]
    pub fn author_js(author: &AuthorId, opts: Option<JsQueryOptions>) -> napi::Result<Self> {
        let opts = opts.map(|o| o.try_into()).transpose()?;
        Ok(Query::author(author, opts))
    }

    /// Query all entries that have an exact key.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(js_name = "keyExact")]
    pub fn key_exact_js(key: Buffer, opts: Option<JsQueryOptions>) -> napi::Result<Self> {
        let opts = opts.map(|o| o.try_into()).transpose()?;
        Ok(Query::key_exact(key.into(), opts))
    }

    /// Create a Query for a single key and author.
    #[napi(js_name = "authorKeyExact")]
    pub fn author_key_exact_js(author: &AuthorId, key: Buffer) -> Self {
        Query::author_key_exact(author, key.into())
    }

    /// Create a query for all entries with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(js_name = "keyPrefix")]
    pub fn key_prefix_js(prefix: Buffer, opts: Option<JsQueryOptions>) -> napi::Result<Self> {
        let opts = opts.map(|o| o.try_into()).transpose()?;
        Ok(Query::key_prefix(prefix.into(), opts))
    }

    /// Create a query for all entries of a single author with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[napi(js_name = "authorKeyPrefix")]
    pub fn author_key_prefix_js(
        author: &AuthorId,
        prefix: Buffer,
        opts: Option<JsQueryOptions>,
    ) -> napi::Result<Self> {
        let opts = opts.map(|o| o.try_into()).transpose()?;
        Ok(Query::author_key_prefix(author, prefix.into(), opts))
    }
}
