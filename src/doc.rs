use std::{str::FromStr, sync::Arc, time::SystemTime};

use futures::{StreamExt, TryStreamExt};
use iroh::{
    client::Doc as ClientDoc,
    rpc_protocol::{ProviderRequest, ProviderResponse},
};
use napi::iterator::Generator;
use napi_derive::napi;
use quic_rpc::transport::flume::FlumeConnection;
use serde::{Deserialize, Serialize};

#[cfg(feature = "napi")]
use napi::bindgen_prelude::Buffer;

use crate::{block_on, AuthorId, Hash, IrohError, IrohNode, PublicKey};

#[napi]
#[derive(Debug)]
pub enum CapabilityKind {
    /// A writable replica.
    Write = 1,
    /// A readable replica.
    Read = 2,
}

impl From<iroh::sync::CapabilityKind> for CapabilityKind {
    fn from(value: iroh::sync::CapabilityKind) -> Self {
        match value {
            iroh::sync::CapabilityKind::Write => Self::Write,
            iroh::sync::CapabilityKind::Read => Self::Read,
        }
    }
}

#[napi]
impl IrohNode {
    /// Create a new doc.
    pub fn doc_create(&self) -> Result<Arc<Doc>, IrohError> {
        block_on(&self.rt(), async {
            let doc = self
                .sync_client
                .docs
                .create()
                .await
                .map_err(IrohError::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.rt().clone(),
            }))
        })
    }

    #[cfg(feature = "napi")]
    #[napi(js_name = "docCreate")]
    pub async fn doc_create_js(&self) -> Result<JsDoc, napi::Error> {
        Ok(JsDoc::new(self).await)
    }

    /// Join and sync with an already existing document.
    pub fn doc_join(&self, ticket: String) -> Result<Arc<Doc>, IrohError> {
        block_on(&self.rt(), async {
            let ticket =
                iroh::ticket::DocTicket::from_str(&ticket).map_err(IrohError::doc_ticket)?;
            let doc = self
                .sync_client
                .docs
                .import(ticket)
                .await
                .map_err(IrohError::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.rt().clone(),
            }))
        })
    }

    /// Join and sync with an already existing document.
    #[cfg(feature = "napi")]
    #[napi(js_name = "docJoin")]
    pub async fn doc_join_js(&self, ticket: String) -> Result<JsDoc, napi::Error> {
        let ticket = iroh::ticket::DocTicket::from_str(&ticket).map_err(anyhow::Error::from)?;
        let doc = self.sync_client.docs.import(ticket).await?;

        Ok(JsDoc { inner: doc })
    }

    /// List all the docs we have access to on this node.
    pub fn doc_list(&self) -> Result<Vec<NamespaceAndCapability>, IrohError> {
        block_on(&self.rt(), async {
            let docs = self
                .sync_client
                .docs
                .list()
                .await
                .map_err(IrohError::doc)?
                .map_ok(|(namespace, capability)| NamespaceAndCapability {
                    namespace: namespace.to_string(),
                    capability: capability.into(),
                })
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::doc)?;

            Ok(docs)
        })
    }

    /// List all the docs we have access to on this node.
    #[cfg(feature = "napi")]
    #[napi(js_name = "docList")]
    pub async fn doc_list_js(&self) -> Result<Vec<NamespaceAndCapability>, napi::Error> {
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
    pub fn doc_open(&self, id: String) -> Result<Option<Arc<Doc>>, IrohError> {
        let namespace_id = iroh::sync::NamespaceId::from_str(&id).map_err(IrohError::namespace)?;
        block_on(&self.rt(), async {
            let doc = self
                .sync_client
                .docs
                .open(namespace_id)
                .await
                .map_err(IrohError::doc)?;
            Ok(doc.map(|d| {
                Arc::new(Doc {
                    inner: d,
                    rt: self.rt().clone(),
                })
            }))
        })
    }

    /// Get a [`Doc`].
    ///
    /// Returns None if the document cannot be found.
    #[cfg(feature = "napi")]
    #[napi(js_name = "docOpen")]
    pub async fn doc_open_js(&self, id: String) -> Result<Option<JsDoc>, napi::Error> {
        let namespace_id = iroh::sync::NamespaceId::from_str(&id)?;
        let doc = self.sync_client.docs.open(namespace_id).await?;

        Ok(doc.map(|d| JsDoc { inner: d }))
    }

    /// Delete a document from the local node.
    ///
    /// This is a destructive operation. Both the document secret key and all entries in the
    /// document will be permanently deleted from the node's storage. Content blobs will be deleted
    /// through garbage collection unless they are referenced from another document or tag.
    pub fn doc_drop(&self, doc_id: String) -> Result<(), IrohError> {
        let doc_id = iroh::sync::NamespaceId::from_str(&doc_id).map_err(IrohError::namespace)?;
        block_on(&self.rt(), async {
            self.sync_client
                .docs
                .drop_doc(doc_id)
                .await
                .map_err(IrohError::doc)
        })
    }

    /// Delete a document from the local node.
    ///
    /// This is a destructive operation. Both the document secret key and all entries in the
    /// document will be permanently deleted from the node's storage. Content blobs will be delted
    /// through garbage collection unless they are referenced from another document or tag.
    #[cfg(feature = "napi")]
    #[napi(js_name = "docDrop")]
    pub async fn doc_drop_js(&self, doc_id: String) -> Result<(), napi::Error> {
        let doc_id = iroh::sync::NamespaceId::from_str(&doc_id)?;
        self.sync_client.docs.drop_doc(doc_id).await?;
        Ok(())
    }
}

/// The namespace id and CapabilityKind (read/write) of the doc
#[napi]
pub struct NamespaceAndCapability {
    /// The namespace id of the doc
    pub namespace: String,
    /// The capability you have for the doc (read/write)
    pub capability: CapabilityKind,
}

/// A representation of a mutable, synchronizable key-value store.
#[derive(Clone)]
pub struct Doc {
    pub(crate) inner: ClientDoc<FlumeConnection<ProviderResponse, ProviderRequest>>,
    pub(crate) rt: tokio::runtime::Handle,
}

impl Doc {
    /// Get the document id of this doc.
    pub fn id(&self) -> String {
        self.inner.id().to_string()
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
        author_id: &AuthorId,
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

    /// Share this document with peers over a ticket.
    pub fn share(&self, mode: ShareMode) -> Result<String, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .share(mode.into())
                .await
                .map(|ticket| ticket.to_string())
                .map_err(IrohError::doc)
        })
    }

    /// Start to sync this document with a list of peers.
    pub fn start_sync(&self, peers: Vec<Arc<NodeAddr>>) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner
                .start_sync(
                    peers
                        .into_iter()
                        .map(|p| (*p).clone().try_into())
                        .collect::<Result<Vec<_>, IrohError>>()?,
                )
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

    /// Subscribe to events for this document.
    pub fn subscribe(&self, cb: Box<dyn SubscribeCallback>) -> Result<(), IrohError> {
        let client = self.inner.clone();
        self.rt.spawn(async move {
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

    /// Set the download policy for this document
    pub fn set_download_policy(&self, policy: Arc<DownloadPolicy>) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner
                .set_download_policy((*policy).clone().into())
                .await
                .map_err(IrohError::doc)
        })
    }

    /// Get the download policy for this document
    pub fn get_download_policy(&self) -> Result<Arc<DownloadPolicy>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .get_download_policy()
                .await
                .map(|policy| Arc::new(policy.into()))
                .map_err(IrohError::doc)
        })
    }
}

/// A representation of a mutable, synchronizable key-value store.
#[cfg(feature = "napi")]
#[napi(js_name = "Doc")]
#[derive(Clone)]
pub struct JsDoc {
    pub(crate) inner: ClientDoc<FlumeConnection<ProviderResponse, ProviderRequest>>,
}

#[cfg(feature = "napi")]
#[napi]
impl JsDoc {
    #[cfg(feature = "napi")]
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
        let (_, size, _) = size.get_u64();
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
        cb: Option<napi::threadsafe_function::ThreadsafeFunction<serde_json::Value>>,
    ) -> Result<(), napi::Error> {
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

        while let Some(progress) = stream.next().await {
            if let Some(ref cb) = cb {
                let progress: DocImportProgress = progress?.into();
                cb.call_async(Ok(serde_json::to_value(progress)?)).await?
            }
        }
        Ok(())
    }

    /// Export an entry as a file to a given absolute path
    #[napi]
    pub async fn export_file(
        &self,
        entry: &Entry,
        path: String,
        cb: Option<napi::threadsafe_function::ThreadsafeFunction<serde_json::Value>>,
    ) -> Result<(), napi::Error> {
        let mut stream = self
            .inner
            .export_file(entry.0.clone(), std::path::PathBuf::from(path))
            .await?;

        while let Some(progress) = stream.next().await {
            if let Some(ref cb) = cb {
                let progress: DocExportProgress = progress?.into();
                cb.call_async::<serde_json::Value>(Ok(serde_json::to_value(progress)?))
                    .await?;
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
    pub async fn del(
        &self,
        author_id: &AuthorId,
        prefix: Buffer,
    ) -> Result<napi::bindgen_prelude::BigInt, napi::Error> {
        let prefix: Vec<_> = prefix.into();
        let num_del = self.inner.del(author_id.0, prefix).await?;

        Ok(u64::try_from(num_del).map_err(anyhow::Error::from)?.into())
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

/// Download policy to decide which content blobs shall be downloaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadPolicy {
    /// Do not download any key unless it matches one of the filters.
    NothingExcept(Vec<Arc<FilterKind>>),
    /// Download every key unless it matches one of the filters.
    EverythingExcept(Vec<Arc<FilterKind>>),
}

impl DownloadPolicy {
    /// Download everything
    pub fn everything() -> Self {
        DownloadPolicy::EverythingExcept(Vec::default())
    }

    /// Download nothing
    pub fn nothing() -> Self {
        DownloadPolicy::NothingExcept(Vec::default())
    }

    /// Download nothing except keys that match the given filters
    pub fn nothing_except(filters: Vec<Arc<FilterKind>>) -> Self {
        DownloadPolicy::NothingExcept(filters)
    }

    /// Download everything except keys that match the given filters
    pub fn everything_except(filters: Vec<Arc<FilterKind>>) -> Self {
        DownloadPolicy::EverythingExcept(filters)
    }
}

impl From<iroh::sync::store::DownloadPolicy> for DownloadPolicy {
    fn from(value: iroh::sync::store::DownloadPolicy) -> Self {
        match value {
            iroh::sync::store::DownloadPolicy::NothingExcept(filters) => {
                DownloadPolicy::NothingExcept(
                    filters.into_iter().map(|f| Arc::new(f.into())).collect(),
                )
            }
            iroh::sync::store::DownloadPolicy::EverythingExcept(filters) => {
                DownloadPolicy::EverythingExcept(
                    filters.into_iter().map(|f| Arc::new(f.into())).collect(),
                )
            }
        }
    }
}

impl From<DownloadPolicy> for iroh::sync::store::DownloadPolicy {
    fn from(value: DownloadPolicy) -> Self {
        match value {
            DownloadPolicy::NothingExcept(filters) => {
                iroh::sync::store::DownloadPolicy::NothingExcept(
                    filters.into_iter().map(|f| f.0.clone()).collect(),
                )
            }
            DownloadPolicy::EverythingExcept(filters) => {
                iroh::sync::store::DownloadPolicy::EverythingExcept(
                    filters.into_iter().map(|f| f.0.clone()).collect(),
                )
            }
        }
    }
}

/// Filter strategy used in download policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterKind(pub(crate) iroh::sync::store::FilterKind);

impl FilterKind {
    /// Verifies whether this filter matches a given key
    pub fn matches(&self, key: Vec<u8>) -> bool {
        self.0.matches(key)
    }

    /// Returns a FilterKind that matches if the contained bytes are a prefix of the key.
    pub fn prefix(prefix: Vec<u8>) -> FilterKind {
        FilterKind(iroh::sync::store::FilterKind::Prefix(bytes::Bytes::from(
            prefix,
        )))
    }

    /// Returns a FilterKind that matches if the contained bytes and the key are the same.
    pub fn exact(key: Vec<u8>) -> FilterKind {
        FilterKind(iroh::sync::store::FilterKind::Exact(bytes::Bytes::from(
            key,
        )))
    }
}

impl From<iroh::sync::store::FilterKind> for FilterKind {
    fn from(value: iroh::sync::store::FilterKind) -> Self {
        FilterKind(value)
    }
}

/// The state for an open replica.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
#[napi]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAddr {
    node_id: Arc<PublicKey>,
    derp_url: Option<String>,
    addresses: Vec<String>,
}

#[napi]
impl NodeAddr {
    /// Create a new [`NodeAddr`] with empty [`AddrInfo`].
    #[napi(constructor)]
    pub fn new(node_id: &PublicKey, derp_url: Option<String>, addresses: Vec<String>) -> Self {
        Self {
            node_id: Arc::new(node_id.clone()),
            derp_url,
            addresses,
        }
    }

    /// Get the direct addresses of this peer.
    #[napi(getter)]
    pub fn direct_addresses(&self) -> Vec<String> {
        self.addresses.clone()
    }

    /// Get the derp region of this peer.
    #[napi(getter)]
    pub fn derp_url(&self) -> Option<String> {
        self.derp_url.clone()
    }

    /// Returns true if both NodeAddr's have the same values
    #[napi]
    pub fn equal(&self, other: &NodeAddr) -> bool {
        self == other
    }
}

impl TryFrom<NodeAddr> for iroh::net::magic_endpoint::NodeAddr {
    type Error = IrohError;
    fn try_from(value: NodeAddr) -> Result<Self, Self::Error> {
        let mut node_addr = iroh::net::magic_endpoint::NodeAddr::new((&*value.node_id).into());
        let addresses = value
            .direct_addresses()
            .into_iter()
            .map(|addr| std::net::SocketAddr::from_str(&addr).map_err(IrohError::socket_addr))
            .collect::<Result<Vec<_>, IrohError>>()?;

        if let Some(derp_url) = value.derp_url() {
            let url = url::Url::parse(&derp_url).map_err(IrohError::url)?;

            node_addr = node_addr.with_derp_url(url);
        }
        node_addr = node_addr.with_direct_addresses(addresses);
        Ok(node_addr)
    }
}

impl From<iroh::net::magic_endpoint::NodeAddr> for NodeAddr {
    fn from(value: iroh::net::magic_endpoint::NodeAddr) -> Self {
        NodeAddr {
            node_id: Arc::new(value.node_id.into()),
            derp_url: value.info.derp_url.map(|url| url.to_string()),
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
#[napi]
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
/// namespace id. Its value is the 32-byte BLAKE3 [`hash`]
/// of the entry's content data, the size of this content data, and a timestamp.
#[napi]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry(pub(crate) iroh::client::Entry);

impl From<iroh::client::Entry> for Entry {
    fn from(e: iroh::client::Entry) -> Self {
        Entry(e)
    }
}

#[napi]
impl Entry {
    /// Get the [`AuthorId`] of this entry.
    #[napi]
    pub fn author(&self) -> Arc<AuthorId> {
        Arc::new(AuthorId(self.0.id().author()))
    }

    /// Get the content_hash of this entry.
    #[napi]
    pub fn content_hash(&self) -> Arc<Hash> {
        Arc::new(Hash(self.0.content_hash()))
    }

    /// Get the content_length of this entry.
    pub fn content_len(&self) -> u64 {
        self.0.content_len()
    }

    /// Get the content_length of this entry.
    #[cfg(feature = "napi")]
    #[napi(js_name = "contentLen")]
    pub fn content_len_js(&self) -> Option<u32> {
        u32::try_from(self.0.content_len()).ok()
    }

    /// Get the key of this entry.
    #[napi]
    pub fn key(&self) -> Vec<u8> {
        self.0.id().key().to_vec()
    }

    /// Get the namespace id of this entry.
    #[napi]
    pub fn namespace(&self) -> String {
        self.0.id().namespace().to_string()
    }

    /// Read all content of an [`Entry`] into a buffer.
    /// This allocates a buffer for the full entry. Use only if you know that the entry you're
    /// reading is small. If not sure, use [`Self::content_len`] and check the size with
    /// before calling [`Self::content_bytes`].
    pub fn content_bytes(&self, doc: Arc<Doc>) -> Result<Vec<u8>, IrohError> {
        block_on(&doc.rt, async {
            self.0
                .content_bytes(&doc.inner)
                .await
                .map(|c| c.to_vec())
                .map_err(IrohError::entry)
        })
    }

    /// Read all content of an [`Entry`] into a buffer.
    /// This allocates a buffer for the full entry. Use only if you know that the entry you're
    /// reading is small. If not sure, use [`Self::content_len`] and check the size with
    /// before calling [`Self::content_bytes`].
    #[cfg(feature = "napi")]
    #[napi(js_name = "contentBytes")]
    pub async fn content_bytes_js(&self, doc: &JsDoc) -> Result<Buffer, napi::Error> {
        let content = self.0.content_bytes(&doc.inner).await.map(|c| c.to_vec())?;
        Ok(content.into())
    }
}

///d Fields by which the query can be sorted
#[napi]
#[cfg_attr(not(feature = "napi"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub enum SortBy {
    /// Sort by key, then author.
    KeyAuthor,
    /// Sort by author, then key.
    #[default]
    AuthorKey,
}

impl From<iroh::sync::store::SortBy> for SortBy {
    fn from(value: iroh::sync::store::SortBy) -> Self {
        match value {
            iroh::sync::store::SortBy::AuthorKey => SortBy::AuthorKey,
            iroh::sync::store::SortBy::KeyAuthor => SortBy::KeyAuthor,
        }
    }
}

impl From<SortBy> for iroh::sync::store::SortBy {
    fn from(value: SortBy) -> Self {
        match value {
            SortBy::AuthorKey => iroh::sync::store::SortBy::AuthorKey,
            SortBy::KeyAuthor => iroh::sync::store::SortBy::KeyAuthor,
        }
    }
}

/// Sort direction
#[napi]
#[cfg_attr(not(feature = "napi"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub enum SortDirection {
    /// Sort ascending
    #[default]
    Asc,
    /// Sort descending
    Desc,
}

impl From<iroh::sync::store::SortDirection> for SortDirection {
    fn from(value: iroh::sync::store::SortDirection) -> Self {
        match value {
            iroh::sync::store::SortDirection::Asc => SortDirection::Asc,
            iroh::sync::store::SortDirection::Desc => SortDirection::Desc,
        }
    }
}

impl From<SortDirection> for iroh::sync::store::SortDirection {
    fn from(value: SortDirection) -> Self {
        match value {
            SortDirection::Asc => iroh::sync::store::SortDirection::Asc,
            SortDirection::Desc => iroh::sync::store::SortDirection::Desc,
        }
    }
}

/// Build a Query to search for an entry or entries in a doc.
///
/// Use this with `QueryOptions` to determine sorting, grouping, and pagination.
#[napi]
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

#[cfg(feature = "napi")]
#[napi(object, js_name = "QueryOptions")]
#[derive(Clone, Debug, Default)]
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
    pub offset: u32,
    /// Limit to limit the pagination.
    ///
    /// When the limit is 0, the limit does not exist.
    pub limit: u32,
}

#[cfg(feature = "napi")]
impl From<JsQueryOptions> for QueryOptions {
    fn from(value: JsQueryOptions) -> Self {
        Self {
            sort_by: value.sort_by,
            direction: value.direction,
            offset: value.offset as u64,
            limit: value.limit as u64,
        }
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
    pub fn all(opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::all();

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

    /// Query all records.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[cfg(feature = "napi")]
    #[napi(js_name = "all")]
    pub fn all_js(opts: Option<JsQueryOptions>) -> Self {
        Query::all(opts.map(|o| o.into()))
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
            builder = builder.sort_direction(opts.direction.into());
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
    #[cfg(feature = "napi")]
    #[napi(js_name = "singleLatestPerKey")]
    pub fn single_latest_per_key_js(opts: Option<JsQueryOptions>) -> Self {
        Query::single_latest_per_key(opts.map(|o| o.into()))
    }

    /// Query all entries for by a single author.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn author(author: &AuthorId, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::author(author.0);

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

    /// Query all entries for by a single author.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[cfg(feature = "napi")]
    #[napi(js_name = "author")]
    pub fn author_js(author: &AuthorId, opts: Option<JsQueryOptions>) -> Self {
        Query::author(author, opts.map(|o| o.into()))
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
    #[cfg(feature = "napi")]
    #[napi(js_name = "keyExact")]
    pub fn key_exact_js(key: Buffer, opts: Option<JsQueryOptions>) -> Self {
        Query::key_exact(key.into(), opts.map(|o| o.into()))
    }

    /// Create a Query for a single key and author.
    pub fn author_key_exact(author: &AuthorId, key: Vec<u8>) -> Self {
        let builder = iroh::sync::store::Query::author(author.0).key_exact(key);
        Query(builder.build())
    }

    /// Create a Query for a single key and author.
    #[cfg(feature = "napi")]
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
    pub fn key_prefix(prefix: Vec<u8>, opts: Option<QueryOptions>) -> Self {
        let mut builder = iroh::sync::store::Query::key_prefix(prefix);

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

    /// Create a query for all entries with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     sort_by: SortBy::AuthorKey
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    #[cfg(feature = "napi")]
    #[napi(js_name = "keyPrefix")]
    pub fn key_prefix_js(prefix: Buffer, opts: Option<JsQueryOptions>) -> Self {
        Query::key_prefix(prefix.into(), opts.map(|o| o.into()))
    }

    /// Create a query for all entries of a single author with a given key prefix.
    ///
    /// If `opts` is `None`, the default values will be used:
    ///     direction: SortDirection::Asc
    ///     offset: None
    ///     limit: None
    pub fn author_key_prefix(
        author: &AuthorId,
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
    #[cfg(feature = "napi")]
    #[napi(js_name = "authorKeyPrefix")]
    pub fn author_key_prefix_js(
        author: &AuthorId,
        prefix: Buffer,
        opts: Option<JsQueryOptions>,
    ) -> Self {
        Query::author_key_prefix(author, prefix.into(), opts.map(|o| o.into()))
    }

    /// Get the limit for this query (max. number of entries to emit).
    pub fn limit(&self) -> Option<u64> {
        self.0.limit()
    }

    /// Get the limit for this query (max. number of entries to emit).
    #[cfg(feature = "napi")]
    #[napi(js_name = "limit")]
    pub fn limit_js(&self) -> Option<u32> {
        match self.0.limit() {
            None => None,
            Some(i) => u32::try_from(i).ok(),
        }
    }

    /// Get the offset for this query (number of entries to skip at the beginning).
    pub fn offset(&self) -> u64 {
        self.0.offset()
    }

    /// Get the limit for this query (max. number of entries to emit).
    #[cfg(feature = "napi")]
    #[napi(js_name = "offset")]
    pub fn offset_js(&self) -> Option<u32> {
        u32::try_from(self.offset()).ok()
    }
}

/// The `progress` method will be called for each `SubscribeProgress` event that is
/// emitted during a `node.doc_subscribe`. Use the `SubscribeProgress.type()`
/// method to check the `LiveEvent`
pub trait SubscribeCallback: Send + Sync + 'static {
    fn event(&self, event: Arc<LiveEvent>) -> Result<(), IrohError>;
}

/// Events informing about actions of the live sync progress
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct InsertRemoteEvent {
    /// The peer that sent us the entry.
    pub from: Arc<PublicKey>,
    /// The inserted entry.
    pub entry: Arc<Entry>,
    /// If the content is available at the local node
    pub content_status: ContentStatus,
}

/// Whether the content status is available on a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocImportProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// A DocImportProgress event indicating we've made progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocImportProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DocImportProgress event indicating we are finished adding `id` to the data store and the hash is `hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocImportProgressIngestDone {
    /// The unique id of the entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// A DocImportProgress event indicating we are done setting the entry to the doc
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocImportProgressAllDone {
    /// The key of the entry
    pub key: Vec<u8>,
}

/// A DocImportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocImportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocExportProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DocExportProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocExportProgressAbort {
    /// The error message
    pub error: String,
}

/// Progress updates for the doc import file operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    use crate::PublicKey;
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
        println!("doc_ticket: {}", doc_ticket);
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
                if let LiveEvent::ContentReady { ref hash } = *event {
                    self.found_s
                        .send(Ok(hash.clone()))
                        .map_err(IrohError::doc)?;
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
            .set_bytes(&author, b"hello".to_vec(), b"world".to_vec())
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

        let got_derp_url = node_addr.derp_url().unwrap();
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
        let hash = doc.set_bytes(&author, key.clone(), val.clone()).unwrap();

        // get entry
        let query = Query::author_key_exact(&author, key.clone());
        let entry = doc.get_one(query.into()).unwrap().unwrap();

        assert!(hash.equal(&entry.content_hash()));

        let got_val = entry.content_bytes(doc).unwrap();
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
            .get_one(Query::author_key_exact(&author, key).into())
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
