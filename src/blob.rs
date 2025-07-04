use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use futures::{StreamExt, TryStreamExt};
use iroh_blobs::store::BaoBlobSize;
use serde::{Deserialize, Serialize};

use crate::{node::Iroh, BlobsClient, CallbackError, NetClient};
use crate::{ticket::AddrInfoOptions, BlobTicket};
use crate::{IrohError, NodeAddr};

/// Iroh blobs client.
#[derive(uniffi::Object)]
pub struct Blobs {
    client: BlobsClient,
    net_client: NetClient,
}

#[uniffi::export]
impl Iroh {
    /// Access to blob specific funtionaliy.
    pub fn blobs(&self) -> Blobs {
        Blobs {
            client: self.blobs_client.clone(),
            net_client: self.net_client.clone(),
        }
    }
}

#[uniffi::export]
impl Blobs {
    /// List all complete blobs.
    ///
    /// Note: this allocates for each `BlobListResponse`, if you have many `BlobListReponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn list(&self) -> Result<Vec<Arc<Hash>>, IrohError> {
        let response = self.client.list().await?;

        let hashes: Vec<Arc<Hash>> = response
            .map_ok(|i| Arc::new(Hash(i.hash)))
            .try_collect()
            .await?;

        Ok(hashes)
    }

    /// Get the size information on a single blob.
    ///
    /// Method only exists in FFI
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn size(&self, hash: &Hash) -> Result<u64, IrohError> {
        let r = self.client.read(hash.0).await?;
        Ok(r.size())
    }

    /// Check if a blob is completely stored on the node.
    ///
    /// This is just a convenience wrapper around `status` that returns a boolean.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn has(&self, hash: &Hash) -> Result<bool, IrohError> {
        let has_blob = self.client.has(hash.0).await?;
        Ok(has_blob)
    }

    /// Check the storage status of a blob on this node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn status(&self, hash: &Hash) -> Result<BlobStatus, IrohError> {
        let status = self.client.status(hash.0).await?;

        Ok(status.into())
    }

    /// Read all bytes of single blob.
    ///
    /// This allocates a buffer for the full blob. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_to_bytes`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_to_bytes(&self, hash: Arc<Hash>) -> Result<Vec<u8>, IrohError> {
        let res = self
            .client
            .read_to_bytes(hash.0)
            .await
            .map(|b| b.to_vec())?;
        Ok(res)
    }

    /// Read all bytes of single blob at `offset` for length `len`.
    ///
    /// This allocates a buffer for the full length `len`. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_at_to_bytes`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_at_to_bytes(
        &self,
        hash: Arc<Hash>,
        offset: u64,
        len: &ReadAtLen,
    ) -> Result<Vec<u8>, IrohError> {
        let res = self
            .client
            .read_at_to_bytes(hash.0, offset, (*len).into())
            .await
            .map(|b| b.to_vec())?;
        Ok(res)
    }

    /// Import a blob from a filesystem path.
    ///
    /// `path` should be an absolute path valid for the file system on which
    /// the node runs.
    /// If `in_place` is true, Iroh will assume that the data will not change and will share it in
    /// place without copying to the Iroh data directory.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_from_path(
        &self,
        path: String,
        in_place: bool,
        tag: Arc<SetTagOption>,
        wrap: Arc<WrapOption>,
        cb: Arc<dyn AddCallback>,
    ) -> Result<(), IrohError> {
        let mut stream = self
            .client
            .add_from_path(
                path.into(),
                in_place,
                (*tag).clone().into(),
                (*wrap).clone().into(),
            )
            .await?;
        while let Some(progress) = stream.next().await {
            let progress = progress?;
            cb.progress(Arc::new(progress.into())).await?;
        }
        Ok(())
    }

    /// Export the blob contents to a file path
    /// The `path` field is expected to be the absolute path.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn write_to_path(&self, hash: Arc<Hash>, path: String) -> Result<(), IrohError> {
        let mut reader = self.client.read(hash.0).await?;
        let path: PathBuf = path.into();
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(anyhow::Error::from)?;
        }
        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(anyhow::Error::from)?;
        tokio::io::copy(&mut reader, &mut file)
            .await
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    /// Write a blob by passing bytes.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_bytes(&self, bytes: Vec<u8>) -> Result<BlobAddOutcome, IrohError> {
        let res = self.client.add_bytes(bytes).await?;
        Ok(res.into())
    }

    /// Write a blob by passing bytes, setting an explicit tag name.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_bytes_named(
        &self,
        bytes: Vec<u8>,
        name: String,
    ) -> Result<BlobAddOutcome, IrohError> {
        let res = self
            .client
            .add_bytes_named(bytes, iroh_blobs::Tag(name.into()))
            .await?;
        Ok(res.into())
    }

    /// Download a blob from another node and add it to the local database.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn download(
        &self,
        hash: Arc<Hash>,
        opts: Arc<BlobDownloadOptions>,
        cb: Arc<dyn DownloadCallback>,
    ) -> Result<(), IrohError> {
        let mut stream = self
            .client
            .download_with_opts(hash.0, opts.0.clone())
            .await?;
        while let Some(progress) = stream.next().await {
            let progress = progress?;
            cb.progress(Arc::new(progress.into())).await?;
        }
        Ok(())
    }

    /// Export a blob from the internal blob store to a path on the node's filesystem.
    ///
    /// `destination` should be a writeable, absolute path on the local node's filesystem.
    ///
    /// If `format` is set to [`ExportFormat::Collection`], and the `hash` refers to a collection,
    /// all children of the collection will be exported. See [`ExportFormat`] for details.
    ///
    /// The `mode` argument defines if the blob should be copied to the target location or moved out of
    /// the internal store into the target location. See [`ExportMode`] for details.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn export(
        &self,
        hash: Arc<Hash>,
        destination: String,
        format: BlobExportFormat,
        mode: BlobExportMode,
    ) -> Result<(), IrohError> {
        let destination: PathBuf = destination.into();
        if let Some(dir) = destination.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(anyhow::Error::from)?;
        }

        let stream = self
            .client
            .export(hash.0, destination, format.into(), mode.into())
            .await?;

        stream.finish().await?;

        Ok(())
    }

    /// Create a ticket for sharing a blob from this node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn share(
        &self,
        hash: Arc<Hash>,
        blob_format: BlobFormat,
        ticket_options: AddrInfoOptions,
    ) -> Result<Arc<BlobTicket>, IrohError> {
        let addr = self.net_client.node_addr().await?;
        let opts: iroh_docs::rpc::AddrInfoOptions = ticket_options.into();
        let addr = opts.apply(&addr);
        let ticket = iroh_blobs::ticket::BlobTicket::new(addr, hash.0, blob_format.into())?;
        Ok(Arc::new(ticket.into()))
    }

    /// List all incomplete (partial) blobs.
    ///
    /// Note: this allocates for each `BlobListIncompleteResponse`, if you have many `BlobListIncompleteResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn list_incomplete(&self) -> Result<Vec<IncompleteBlobInfo>, IrohError> {
        let blobs = self
            .client
            .list_incomplete()
            .await?
            .map_ok(|res| res.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(blobs)
    }

    /// List all collections.
    ///
    /// Note: this allocates for each `BlobListCollectionsResponse`, if you have many `BlobListCollectionsResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>, IrohError> {
        let blobs = self
            .client
            .list_collections()?
            .map_ok(|res| res.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(blobs)
    }

    /// Read the content of a collection
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_collection(&self, hash: Arc<Hash>) -> Result<Arc<Collection>, IrohError> {
        let collection = self.client.get_collection(hash.0).await?;

        Ok(Arc::new(collection.into()))
    }

    /// Create a collection from already existing blobs.
    ///
    /// To automatically clear the tags for the passed in blobs you can set
    /// `tags_to_delete` on those tags, and they will be deleted once the collection is created.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn create_collection(
        &self,
        collection: Arc<Collection>,
        tag: Arc<SetTagOption>,
        tags_to_delete: Vec<String>,
    ) -> Result<HashAndTag, IrohError> {
        let collection = collection.0.read().unwrap().clone();
        let (hash, tag) = self
            .client
            .create_collection(
                collection,
                (*tag).clone().into(),
                tags_to_delete
                    .into_iter()
                    .map(iroh_blobs::Tag::from)
                    .collect(),
            )
            .await?;

        Ok(HashAndTag {
            hash: Arc::new(hash.into()),
            tag: tag.0.to_vec(),
        })
    }

    /// Delete a blob.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn delete_blob(&self, hash: Arc<Hash>) -> Result<(), IrohError> {
        let mut tags = self.client.tags().list().await?;

        let mut name = None;
        while let Some(tag) = tags.next().await {
            let tag = tag?;
            if tag.hash == hash.0 {
                name = Some(tag.name);
            }
        }

        if let Some(name) = name {
            self.client.tags().delete(name).await?;
            self.client.delete_blob((*hash).clone().0).await?;
        }

        Ok(())
    }
}

/// The Hash and associated tag of a newly created collection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct HashAndTag {
    /// The hash of the collection
    pub hash: Arc<Hash>,
    /// The tag of the collection
    pub tag: Vec<u8>,
}

/// Outcome of a blob add operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct BlobAddOutcome {
    /// The hash of the blob
    pub hash: Arc<Hash>,
    /// The format the blob
    pub format: BlobFormat,
    /// The size of the blob
    pub size: u64,
    /// The tag of the blob
    pub tag: Vec<u8>,
}

impl From<iroh_blobs::rpc::client::blobs::AddOutcome> for BlobAddOutcome {
    fn from(value: iroh_blobs::rpc::client::blobs::AddOutcome) -> Self {
        BlobAddOutcome {
            hash: Arc::new(value.hash.into()),
            format: value.format.into(),
            size: value.size,
            tag: value.tag.0.to_vec(),
        }
    }
}

/// Status information about a blob.
#[derive(Debug, uniffi::Object, Clone, Copy)]
pub enum BlobStatus {
    /// The blob is not stored at all.
    NotFound,
    /// The blob is only stored partially.
    Partial {
        /// The size of the currently stored partial blob.
        size: u64,
        /// If the size is verified.
        size_is_verified: bool,
    },
    /// The blob is stored completely.
    Complete {
        /// The size of the blob.
        size: u64,
    },
}

impl From<iroh_blobs::rpc::client::blobs::BlobStatus> for BlobStatus {
    fn from(value: iroh_blobs::rpc::client::blobs::BlobStatus) -> Self {
        match value {
            iroh_blobs::rpc::client::blobs::BlobStatus::NotFound => Self::NotFound,
            iroh_blobs::rpc::client::blobs::BlobStatus::Partial { size } => match size {
                BaoBlobSize::Unverified(size) => Self::Partial {
                    size,
                    size_is_verified: false,
                },
                BaoBlobSize::Verified(size) => Self::Partial {
                    size,
                    size_is_verified: true,
                },
            },
            iroh_blobs::rpc::client::blobs::BlobStatus::Complete { size } => {
                Self::Complete { size }
            }
        }
    }
}

/// Defines the way to read bytes.
#[derive(Debug, uniffi::Object, Default, Clone, Copy)]
pub enum ReadAtLen {
    /// Reads all available bytes.
    #[default]
    All,
    /// Reads exactly this many bytes, erroring out on larger or smaller.
    Exact(u64),
    /// Reads at most this many bytes.
    AtMost(u64),
}

#[uniffi::export]
impl ReadAtLen {
    #[uniffi::constructor]
    pub fn all() -> Self {
        Self::All
    }

    #[uniffi::constructor]
    pub fn exact(size: u64) -> Self {
        Self::Exact(size)
    }

    #[uniffi::constructor]
    pub fn at_most(size: u64) -> Self {
        Self::AtMost(size)
    }
}

impl From<ReadAtLen> for iroh_blobs::rpc::client::blobs::ReadAtLen {
    fn from(value: ReadAtLen) -> Self {
        match value {
            ReadAtLen::All => iroh_blobs::rpc::client::blobs::ReadAtLen::All,
            ReadAtLen::Exact(s) => iroh_blobs::rpc::client::blobs::ReadAtLen::Exact(s),
            ReadAtLen::AtMost(s) => iroh_blobs::rpc::client::blobs::ReadAtLen::AtMost(s),
        }
    }
}

/// An option for commands that allow setting a Tag
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub enum SetTagOption {
    /// A tag will be automatically generated
    Auto,
    /// The tag is explicitly vecnamed
    Named(Vec<u8>),
}

#[uniffi::export]
impl SetTagOption {
    /// Indicate you want an automatically generated tag
    #[uniffi::constructor]
    pub fn auto() -> Self {
        SetTagOption::Auto
    }

    /// Indicate you want a named tag
    #[uniffi::constructor]
    pub fn named(tag: Vec<u8>) -> Self {
        SetTagOption::Named(tag)
    }
}

impl From<SetTagOption> for iroh_blobs::util::SetTagOption {
    fn from(value: SetTagOption) -> Self {
        match value {
            SetTagOption::Auto => iroh_blobs::util::SetTagOption::Auto,
            SetTagOption::Named(tag) => {
                iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(bytes::Bytes::from(tag)))
            }
        }
    }
}

/// Whether to wrap the added data in a collection.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub enum WrapOption {
    /// Do not wrap the file or directory.
    NoWrap,
    /// Wrap the file or directory in a colletion.
    Wrap {
        /// Override the filename in the wrapping collection.
        name: Option<String>,
    },
}

#[uniffi::export]
impl WrapOption {
    /// Indicate you do not wrap the file or directory.
    #[uniffi::constructor]
    pub fn no_wrap() -> Self {
        WrapOption::NoWrap
    }

    /// Indicate you want to wrap the file or directory in a colletion, with an optional name
    #[uniffi::constructor]
    pub fn wrap(name: Option<String>) -> Self {
        WrapOption::Wrap { name }
    }
}

impl From<WrapOption> for iroh_blobs::rpc::client::blobs::WrapOption {
    fn from(value: WrapOption) -> Self {
        match value {
            WrapOption::NoWrap => iroh_blobs::rpc::client::blobs::WrapOption::NoWrap,
            WrapOption::Wrap { name } => iroh_blobs::rpc::client::blobs::WrapOption::Wrap { name },
        }
    }
}

/// Hash type used throughout Iroh. A blake3 hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
#[uniffi::export(Display)]
pub struct Hash(pub(crate) iroh_blobs::Hash);

impl From<iroh_blobs::Hash> for Hash {
    fn from(h: iroh_blobs::Hash) -> Self {
        Hash(h)
    }
}

#[uniffi::export]
impl Hash {
    /// Calculate the hash of the provide bytes.
    #[uniffi::constructor]
    pub fn new(buf: Vec<u8>) -> Self {
        Hash(iroh_blobs::Hash::new(buf))
    }

    /// Bytes of the hash.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Create a `Hash` from its raw bytes representation.
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        let bytes: [u8; 32] = bytes.try_into().map_err(|b: Vec<u8>| {
            anyhow::anyhow!("expected byte array of length 32, got {}", b.len())
        })?;
        Ok(Hash(iroh_blobs::Hash::from_bytes(bytes)))
    }

    /// Make a Hash from hex string
    #[uniffi::constructor]
    pub fn from_string(s: String) -> Result<Self, IrohError> {
        let key = iroh_blobs::Hash::from_str(&s).map_err(anyhow::Error::from)?;
        Ok(key.into())
    }

    /// Convert the hash to a hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns true if the Hash's have the same value
    pub fn equal(&self, other: &Hash) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Hash> for iroh_blobs::Hash {
    fn from(value: Hash) -> Self {
        value.0
    }
}

// /// Hash type used throughout Iroh. A blake3 hash.
// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
// #[uniffi::export(Display)]
// pub struct Tag(pub(crate) iroh_blobs::Tag);

// impl From<iroh_blobs::Tag> for Tag {
//     fn from(h: iroh_blobs::Tag) -> Self {
//         Tag(h)
//     }
// }

/// The `progress` method will be called for each `BlobProvideEvent` event that is
/// emitted from the iroh node while the callback is registered. Use the `BlobProvideEvent.type()`
/// method to check the `BlobProvideEventType`
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait BlobProvideEventCallback: Send + Sync + 'static {
    async fn blob_event(&self, event: Arc<BlobProvideEvent>) -> Result<(), CallbackError>;
}

/// The different types of BlobProvide events
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, uniffi::Enum)]
pub enum BlobProvideEventType {
    /// A new collection or tagged blob has been added
    TaggedBlobAdded,
    /// A new client connected to the node.
    ClientConnected,
    /// A request was received from a client.
    GetRequestReceived,
    /// A sequence of hashes has been found and is being transferred.
    TransferHashSeqStarted,
    /// A chunk of a blob was transferred.
    ///
    /// it is not safe to assume all progress events will be sent
    TransferProgress,
    /// A blob in a sequence was transferred.
    TransferBlobCompleted,
    /// A request was completed and the data was sent to the client.
    TransferCompleted,
    /// A request was aborted because the client disconnected.
    TransferAborted,
}

/// An BlobProvide event indicating a new tagged blob or collection was added
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct TaggedBlobAdded {
    /// The hash of the added data
    pub hash: Arc<Hash>,
    /// The format of the added data
    pub format: BlobFormat,
    /// The tag of the added data
    pub tag: Vec<u8>,
}

/// A new client connected to the node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ClientConnected {
    /// An unique connection id.
    pub connection_id: u64,
}

/// A request was received from a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct GetRequestReceived {
    /// An unique connection id.
    pub connection_id: u64,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: u64,
    /// The hash for which the client wants to receive data.
    pub hash: Arc<Hash>,
}

/// A sequence of hashes has been found and is being transferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct TransferHashSeqStarted {
    /// An unique connection id.
    pub connection_id: u64,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: u64,
    /// The number of blobs in the sequence.
    pub num_blobs: u64,
}

/// A chunk of a blob was transferred.
///
/// These events will be sent with try_send, so you can not assume that you
/// will receive all of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct TransferProgress {
    /// An unique connection id.
    pub connection_id: u64,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: u64,
    /// The hash for which we are transferring data.
    pub hash: Arc<Hash>,
    /// Offset up to which we have transferred data.
    pub end_offset: u64,
}

/// A blob in a sequence was transferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct TransferBlobCompleted {
    /// An unique connection id.
    pub connection_id: u64,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: u64,
    /// The hash of the blob
    pub hash: Arc<Hash>,
    /// The index of the blob in the sequence.
    pub index: u64,
    /// The size of the blob transferred.
    pub size: u64,
}

/// A request was completed and the data was sent to the client.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct TransferCompleted {
    /// An unique connection id.
    pub connection_id: u64,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: u64,
    /// statistics about the transfer
    pub stats: TransferStats,
}

/// A request was aborted because the client disconnected.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct TransferAborted {
    /// The quic connection id.
    pub connection_id: u64,
    /// An identifier uniquely identifying this request.
    pub request_id: u64,
    /// statistics about the transfer. This is None if the transfer
    /// was aborted before any data was sent.
    pub stats: Option<TransferStats>,
}

/// The stats for a transfer of a collection or blob.
#[derive(Debug, Clone, Copy, Default, PartialEq, uniffi::Record)]
pub struct TransferStats {
    // /// Stats for sending to the client.
    // pub send: StreamWriterStats,
    // /// Stats for reading from disk.
    // pub read: SliceReaderStats,
    /// The total duration of the transfer in milliseconds
    pub duration: u64,
}

impl From<&iroh_blobs::provider::TransferStats> for TransferStats {
    fn from(value: &iroh_blobs::provider::TransferStats) -> Self {
        Self {
            duration: value
                .duration
                .as_millis()
                .try_into()
                .expect("duration too large"),
        }
    }
}

/// Events emitted by the provider informing about the current status.
#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub enum BlobProvideEvent {
    /// A new collection or tagged blob has been added
    TaggedBlobAdded(TaggedBlobAdded),
    /// A new client connected to the node.
    ClientConnected(ClientConnected),
    /// A request was received from a client.
    GetRequestReceived(GetRequestReceived),
    /// A sequence of hashes has been found and is being transferred.
    TransferHashSeqStarted(TransferHashSeqStarted),
    /// A chunk of a blob was transferred.
    ///
    /// These events will be sent with try_send, so you can not assume that you
    /// will receive all of them.
    TransferProgress(TransferProgress),
    /// A blob in a sequence was transferred.
    TransferBlobCompleted(TransferBlobCompleted),
    /// A request was completed and the data was sent to the client.
    TransferCompleted(TransferCompleted),
    /// A request was aborted because the client disconnected.
    TransferAborted(TransferAborted),
}

impl From<iroh_blobs::provider::Event> for BlobProvideEvent {
    fn from(value: iroh_blobs::provider::Event) -> Self {
        match value {
            iroh_blobs::provider::Event::TaggedBlobAdded { hash, format, tag } => {
                BlobProvideEvent::TaggedBlobAdded(TaggedBlobAdded {
                    hash: Arc::new(hash.into()),
                    format: format.into(),
                    tag: tag.0.as_ref().to_vec(),
                })
            }
            iroh_blobs::provider::Event::ClientConnected { connection_id } => {
                BlobProvideEvent::ClientConnected(ClientConnected { connection_id })
            }
            iroh_blobs::provider::Event::GetRequestReceived {
                connection_id,
                request_id,
                hash,
            } => BlobProvideEvent::GetRequestReceived(GetRequestReceived {
                connection_id,
                request_id,
                hash: Arc::new(hash.into()),
            }),
            iroh_blobs::provider::Event::TransferHashSeqStarted {
                connection_id,
                request_id,
                num_blobs,
            } => BlobProvideEvent::TransferHashSeqStarted(TransferHashSeqStarted {
                connection_id,
                request_id,
                num_blobs,
            }),
            iroh_blobs::provider::Event::TransferProgress {
                connection_id,
                request_id,
                hash,
                end_offset,
            } => BlobProvideEvent::TransferProgress(TransferProgress {
                connection_id,
                request_id,
                hash: Arc::new(hash.into()),
                end_offset,
            }),
            iroh_blobs::provider::Event::TransferBlobCompleted {
                connection_id,
                request_id,
                hash,
                index,
                size,
            } => BlobProvideEvent::TransferBlobCompleted(TransferBlobCompleted {
                connection_id,
                request_id,
                hash: Arc::new(hash.into()),
                index,
                size,
            }),
            iroh_blobs::provider::Event::TransferCompleted {
                connection_id,
                request_id,
                stats,
            } => BlobProvideEvent::TransferCompleted(TransferCompleted {
                connection_id,
                request_id,
                stats: stats.as_ref().into(),
            }),
            iroh_blobs::provider::Event::TransferAborted {
                connection_id,
                request_id,
                stats,
            } => BlobProvideEvent::TransferAborted(TransferAborted {
                connection_id,
                request_id,
                stats: stats.map(|s| s.as_ref().into()),
            }),
        }
    }
}

#[uniffi::export]
impl BlobProvideEvent {
    /// Get the type of event
    pub fn r#type(&self) -> BlobProvideEventType {
        match self {
            BlobProvideEvent::TaggedBlobAdded(_) => BlobProvideEventType::TaggedBlobAdded,
            BlobProvideEvent::ClientConnected(_) => BlobProvideEventType::ClientConnected,
            BlobProvideEvent::GetRequestReceived(_) => BlobProvideEventType::GetRequestReceived,
            BlobProvideEvent::TransferHashSeqStarted(_) => {
                BlobProvideEventType::TransferHashSeqStarted
            }
            BlobProvideEvent::TransferProgress(_) => BlobProvideEventType::TransferProgress,
            BlobProvideEvent::TransferBlobCompleted(_) => {
                BlobProvideEventType::TransferBlobCompleted
            }
            BlobProvideEvent::TransferCompleted(_) => BlobProvideEventType::TransferCompleted,
            BlobProvideEvent::TransferAborted(_) => BlobProvideEventType::TransferAborted,
        }
    }
    /// Return the `TaggedBlobAdded` event
    pub fn as_tagged_blob_added(&self) -> TaggedBlobAdded {
        match self {
            BlobProvideEvent::TaggedBlobAdded(t) => t.clone(),
            _ => panic!("BlobProvideEvent type is not 'TaggedBlobAdded'"),
        }
    }

    /// Return the `ClientConnected` event
    pub fn as_client_connected(&self) -> ClientConnected {
        match self {
            BlobProvideEvent::ClientConnected(c) => c.clone(),
            _ => panic!("BlobProvideEvent type is not 'ClientConnected'"),
        }
    }
    /// Return the `GetRequestReceived` event
    pub fn as_get_request_received(&self) -> GetRequestReceived {
        match self {
            BlobProvideEvent::GetRequestReceived(g) => g.clone(),
            _ => panic!("BlobProvideEvent type is not 'GetRequestReceived'"),
        }
    }
    /// Return the `TransferHashSeqStarted` event
    pub fn as_transfer_hash_seq_started(&self) -> TransferHashSeqStarted {
        match self {
            BlobProvideEvent::TransferHashSeqStarted(t) => t.clone(),
            _ => panic!("BlobProvideEvent type is not 'TransferHashSeqStarted'"),
        }
    }
    /// Return the `TransferProgress` event
    pub fn as_transfer_progress(&self) -> TransferProgress {
        match self {
            BlobProvideEvent::TransferProgress(t) => t.clone(),
            _ => panic!("BlobProvideEvent type is not 'TransferProgress'"),
        }
    }
    /// Return the `TransferBlobCompleted` event
    pub fn as_transfer_blob_completed(&self) -> TransferBlobCompleted {
        match self {
            BlobProvideEvent::TransferBlobCompleted(t) => t.clone(),
            _ => panic!("BlobProvideEvent type is not 'TransferBlobCompleted'"),
        }
    }
    /// Return the `TransferCompleted` event
    pub fn as_transfer_completed(&self) -> TransferCompleted {
        match self {
            BlobProvideEvent::TransferCompleted(t) => t.clone(),
            _ => panic!("BlobProvideEvent type is not 'TransferCompleted'"),
        }
    }
    /// Return the `TransferAborted` event
    pub fn as_transfer_aborted(&self) -> TransferAborted {
        match self {
            BlobProvideEvent::TransferAborted(t) => t.clone(),
            _ => panic!("BlobProvideEvent type is not 'TransferAborted'"),
        }
    }
}

/// The `progress` method will be called for each `AddProgress` event that is
/// emitted during a `node.blobs_add_from_path`. Use the `AddProgress.type()`
/// method to check the `AddProgressType`
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AddCallback: Send + Sync + 'static {
    async fn progress(&self, progress: Arc<AddProgress>) -> Result<(), CallbackError>;
}

/// The different types of AddProgress events
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, uniffi::Enum)]
pub enum AddProgressType {
    /// An item was found with name `name`, from now on referred to via `id`
    Found,
    /// We got progress ingesting item `id`.
    Progress,
    /// We are done with `id`, and the hash is `hash`.
    Done,
    /// We are done with the whole operation.
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort,
}

/// An AddProgress event indicating an item was found with name `name`, that can be referred to by `id`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AddProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// An AddProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AddProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// An AddProgress event indicated we are done with `id` and now have a hash `hash`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AddProgressDone {
    /// The unique id of the entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// An AddProgress event indicating we are done with the the whole operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AddProgressAllDone {
    /// The hash of the created data.
    pub hash: Arc<Hash>,
    /// The format of the added data.
    pub format: BlobFormat,
    /// The tag of the added data.
    pub tag: Vec<u8>,
}

/// An AddProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AddProgressAbort {
    pub error: String,
}

/// Progress updates for the add operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
pub enum AddProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    Found(AddProgressFound),
    /// We got progress ingesting item `id`.
    Progress(AddProgressProgress),
    /// We are done with `id`, and the hash is `hash`.
    Done(AddProgressDone),
    /// We are done with the whole operation.
    AllDone(AddProgressAllDone),
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort(AddProgressAbort),
}

impl From<iroh_blobs::provider::AddProgress> for AddProgress {
    fn from(value: iroh_blobs::provider::AddProgress) -> Self {
        match value {
            iroh_blobs::provider::AddProgress::Found { id, name, size } => {
                AddProgress::Found(AddProgressFound { id, name, size })
            }
            iroh_blobs::provider::AddProgress::Progress { id, offset } => {
                AddProgress::Progress(AddProgressProgress { id, offset })
            }
            iroh_blobs::provider::AddProgress::Done { id, hash } => {
                AddProgress::Done(AddProgressDone {
                    id,
                    hash: Arc::new(hash.into()),
                })
            }
            iroh_blobs::provider::AddProgress::AllDone { hash, format, tag } => {
                AddProgress::AllDone(AddProgressAllDone {
                    hash: Arc::new(hash.into()),
                    format: format.into(),
                    tag: tag.0.to_vec(),
                })
            }
            iroh_blobs::provider::AddProgress::Abort(err) => AddProgress::Abort(AddProgressAbort {
                error: err.to_string(),
            }),
        }
    }
}

#[uniffi::export]
impl AddProgress {
    /// Get the type of event
    pub fn r#type(&self) -> AddProgressType {
        match self {
            AddProgress::Found(_) => AddProgressType::Found,
            AddProgress::Progress(_) => AddProgressType::Progress,
            AddProgress::Done(_) => AddProgressType::Done,
            AddProgress::AllDone(_) => AddProgressType::AllDone,
            AddProgress::Abort(_) => AddProgressType::Abort,
        }
    }
    /// Return the `AddProgressFound` event
    pub fn as_found(&self) -> AddProgressFound {
        match self {
            AddProgress::Found(f) => f.clone(),
            _ => panic!("AddProgress type is not 'Found'"),
        }
    }
    /// Return the `AddProgressProgress` event
    pub fn as_progress(&self) -> AddProgressProgress {
        match self {
            AddProgress::Progress(p) => p.clone(),
            _ => panic!("AddProgress type is not 'Progress'"),
        }
    }

    /// Return the `AddProgressDone` event
    pub fn as_done(&self) -> AddProgressDone {
        match self {
            AddProgress::Done(d) => d.clone(),
            _ => panic!("AddProgress type is not 'Done'"),
        }
    }

    /// Return the `AddAllDone`
    pub fn as_all_done(&self) -> AddProgressAllDone {
        match self {
            AddProgress::AllDone(a) => a.clone(),
            _ => panic!("AddProgress type is not 'AllDone'"),
        }
    }

    /// Return the `AddProgressAbort`
    pub fn as_abort(&self) -> AddProgressAbort {
        match self {
            AddProgress::Abort(a) => a.clone(),
            _ => panic!("AddProgress type is not 'Abort'"),
        }
    }
}

/// A format identifier
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum BlobFormat {
    /// Raw blob
    Raw,
    /// A sequence of BLAKE3 hashes
    HashSeq,
}

impl From<iroh_blobs::BlobFormat> for BlobFormat {
    fn from(value: iroh_blobs::BlobFormat) -> Self {
        match value {
            iroh_blobs::BlobFormat::Raw => BlobFormat::Raw,
            iroh_blobs::BlobFormat::HashSeq => BlobFormat::HashSeq,
        }
    }
}

impl From<BlobFormat> for iroh_blobs::BlobFormat {
    fn from(value: BlobFormat) -> Self {
        match value {
            BlobFormat::Raw => iroh_blobs::BlobFormat::Raw,
            BlobFormat::HashSeq => iroh_blobs::BlobFormat::HashSeq,
        }
    }
}

/// Options to download  data specified by the hash.
#[derive(Debug, uniffi::Object)]
pub struct BlobDownloadOptions(iroh_blobs::rpc::client::blobs::DownloadOptions);

#[uniffi::export]
impl BlobDownloadOptions {
    /// Create a BlobDownloadRequest
    #[uniffi::constructor]
    pub fn new(
        format: BlobFormat,
        nodes: Vec<Arc<NodeAddr>>,
        tag: Arc<SetTagOption>,
    ) -> Result<Self, IrohError> {
        Ok(BlobDownloadOptions(
            iroh_blobs::rpc::client::blobs::DownloadOptions {
                format: format.into(),
                nodes: nodes
                    .into_iter()
                    .map(|node| (*node).clone().try_into())
                    .collect::<Result<_, _>>()?,
                tag: (*tag).clone().into(),
                mode: iroh_blobs::rpc::client::blobs::DownloadMode::Direct,
            },
        ))
    }
}

impl From<iroh_blobs::rpc::client::blobs::DownloadOptions> for BlobDownloadOptions {
    fn from(value: iroh_blobs::rpc::client::blobs::DownloadOptions) -> Self {
        BlobDownloadOptions(value)
    }
}

/// The expected format of a hash being exported.
#[derive(Debug, uniffi::Enum)]
pub enum BlobExportFormat {
    /// The hash refers to any blob and will be exported to a single file.
    Blob,
    /// The hash refers to a [`crate::format::collection::Collection`] blob
    /// and all children of the collection shall be exported to one file per child.
    ///
    /// If the blob can be parsed as a [`BlobFormat::HashSeq`], and the first child contains
    /// collection metadata, all other children of the collection will be exported to
    /// a file each, with their collection name treated as a relative path to the export
    /// destination path.
    ///
    /// If the blob cannot be parsed as a collection, the operation will fail.
    Collection,
}

impl From<BlobExportFormat> for iroh_blobs::store::ExportFormat {
    fn from(value: BlobExportFormat) -> Self {
        match value {
            BlobExportFormat::Blob => iroh_blobs::store::ExportFormat::Blob,
            BlobExportFormat::Collection => iroh_blobs::store::ExportFormat::Collection,
        }
    }
}

/// The export mode describes how files will be exported.
///
/// This is a hint to the import trait method. For some implementations, this
/// does not make any sense. E.g. an in memory implementation will always have
/// to copy the file into memory. Also, a disk based implementation might choose
/// to copy small files even if the mode is `Reference`.
#[derive(Debug, uniffi::Enum)]
pub enum BlobExportMode {
    /// This mode will copy the file to the target directory.
    ///
    /// This is the safe default because the file can not be accidentally modified
    /// after it has been exported.
    Copy,
    /// This mode will try to move the file to the target directory and then reference it from
    /// the database.
    ///
    /// This has a large performance and storage benefit, but it is less safe since
    /// the file might be modified in the target directory after it has been exported.
    ///
    /// Stores are allowed to ignore this mode and always copy the file, e.g.
    /// if the file is very small or if the store does not support referencing files.
    TryReference,
}

impl From<BlobExportMode> for iroh_blobs::store::ExportMode {
    fn from(value: BlobExportMode) -> Self {
        match value {
            BlobExportMode::Copy => iroh_blobs::store::ExportMode::Copy,
            BlobExportMode::TryReference => iroh_blobs::store::ExportMode::TryReference,
        }
    }
}

/// The `progress` method will be called for each `DownloadProgress` event that is emitted during
/// a `node.blobs_download`. Use the `DownloadProgress.type()` method to check the
/// `DownloadProgressType` of the event.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait DownloadCallback: Send + Sync + 'static {
    async fn progress(&self, progress: Arc<DownloadProgress>) -> Result<(), CallbackError>;
}

/// The different types of DownloadProgress events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum DownloadProgressType {
    InitialState,
    FoundLocal,
    Connected,
    Found,
    FoundHashSeq,
    Progress,
    Done,
    AllDone,
    Abort,
}

/// A DownloadProgress event indicating an item was found with hash `hash`, that can be referred to by `id`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// child offset
    pub child: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// A DownloadProgress event indicating an entry was found locally
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressFoundLocal {
    /// child offset
    pub child: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
    /// The size of the entry in bytes.
    pub size: u64,
    /// The ranges that are available locally.
    pub valid_ranges: Arc<RangeSpec>,
}

/// A DownloadProgress event indicating an item was found with hash `hash`, that can be referred to by `id`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressFoundHashSeq {
    /// Number of children in the collection, if known.
    pub children: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// A DownloadProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DownloadProgress event indicated we are done with `id`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressDone {
    /// The unique id of the entry.
    pub id: u64,
}

/// A DownloadProgress event indicating we are done with the whole operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressAllDone {
    /// The number of bytes written
    pub bytes_written: u64,
    /// The number of bytes read
    pub bytes_read: u64,
    /// The time it took to transfer the data
    pub elapsed: Duration,
}

/// A DownloadProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressAbort {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadProgressInitialState {
    // TODO(b5) - numerous fields missing
    // /// The root blob of this transfer (may be a hash seq),
    // pub root: BlobState,
    /// Whether we are connected to a node
    pub connected: bool,
    // /// Children if the root blob is a hash seq, empty for raw blobs
    // pub children: HashMap<NonZeroU64, BlobState>,
    // /// Child being transferred at the moment.
    // pub current: Option<BlobId>,
    // /// Progress ids for individual blobs.
    // pub progress_id_to_blob: HashMap<ProgressId, BlobId>,
}

/// Progress updates for the get operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
pub enum DownloadProgress {
    /// Initial state if subscribing to a running or queued transfer.
    InitialState(DownloadProgressInitialState),
    /// A new connection was established.
    Connected,
    /// An item was found with hash `hash`, from now on referred to via `id`
    Found(DownloadProgressFound),
    /// Data was found locally
    FoundLocal(DownloadProgressFoundLocal),
    /// An item was found with hash `hash`, from now on referred to via `id`
    FoundHashSeq(DownloadProgressFoundHashSeq),
    /// We got progress ingesting item `id`.
    Progress(DownloadProgressProgress),
    /// We are done with `id`, and the hash is `hash`.
    Done(DownloadProgressDone),
    /// We are done with the whole operation.
    AllDone(DownloadProgressAllDone),
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort(DownloadProgressAbort),
}

impl From<iroh_blobs::get::db::DownloadProgress> for DownloadProgress {
    fn from(value: iroh_blobs::get::db::DownloadProgress) -> Self {
        match value {
            iroh_blobs::get::db::DownloadProgress::InitialState(transfer_state) => {
                DownloadProgress::InitialState(DownloadProgressInitialState {
                    connected: transfer_state.connected,
                })
            }
            iroh_blobs::get::db::DownloadProgress::FoundLocal {
                child,
                hash,
                size,
                valid_ranges,
            } => DownloadProgress::FoundLocal(DownloadProgressFoundLocal {
                child: child.into(),
                hash: Arc::new(hash.into()),
                // TODO(b5) - this is ignoring verification information!
                size: size.value(),
                valid_ranges: Arc::new(valid_ranges.into()),
            }),
            iroh_blobs::get::db::DownloadProgress::Connected => DownloadProgress::Connected,
            iroh_blobs::get::db::DownloadProgress::Found {
                id,
                hash,
                child,
                size,
            } => DownloadProgress::Found(DownloadProgressFound {
                id,
                hash: Arc::new(hash.into()),
                child: child.into(),
                size,
            }),
            iroh_blobs::get::db::DownloadProgress::FoundHashSeq { hash, children } => {
                DownloadProgress::FoundHashSeq(DownloadProgressFoundHashSeq {
                    hash: Arc::new(hash.into()),
                    children,
                })
            }
            iroh_blobs::get::db::DownloadProgress::Progress { id, offset } => {
                DownloadProgress::Progress(DownloadProgressProgress { id, offset })
            }
            iroh_blobs::get::db::DownloadProgress::Done { id } => {
                DownloadProgress::Done(DownloadProgressDone { id })
            }
            iroh_blobs::get::db::DownloadProgress::AllDone(stats) => {
                DownloadProgress::AllDone(DownloadProgressAllDone {
                    bytes_written: stats.bytes_written,
                    bytes_read: stats.bytes_read,
                    elapsed: stats.elapsed,
                })
            }
            iroh_blobs::get::db::DownloadProgress::Abort(err) => {
                DownloadProgress::Abort(DownloadProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

#[uniffi::export]
impl DownloadProgress {
    /// Get the type of event
    /// note that there is no `as_connected` method, as the `Connected` event has no associated data
    pub fn r#type(&self) -> DownloadProgressType {
        match self {
            DownloadProgress::InitialState(_) => DownloadProgressType::InitialState,
            DownloadProgress::Connected => DownloadProgressType::Connected,
            DownloadProgress::Found(_) => DownloadProgressType::Found,
            DownloadProgress::FoundLocal(_) => DownloadProgressType::FoundLocal,
            DownloadProgress::FoundHashSeq(_) => DownloadProgressType::FoundHashSeq,
            DownloadProgress::Progress(_) => DownloadProgressType::Progress,
            DownloadProgress::Done(_) => DownloadProgressType::Done,
            DownloadProgress::AllDone(_) => DownloadProgressType::AllDone,
            DownloadProgress::Abort(_) => DownloadProgressType::Abort,
        }
    }

    /// Return the `DownloadProgressFound` event
    pub fn as_found(&self) -> DownloadProgressFound {
        match self {
            DownloadProgress::Found(f) => f.clone(),
            _ => panic!("DownloadProgress type is not 'Found'"),
        }
    }

    /// Return the `DownloadProgressFoundLocal` event
    pub fn as_found_local(&self) -> DownloadProgressFoundLocal {
        match self {
            DownloadProgress::FoundLocal(f) => f.clone(),
            _ => panic!("DownloadProgress type is not 'FoundLocal'"),
        }
    }

    /// Return the `DownloadProgressFoundHashSeq` event
    pub fn as_found_hash_seq(&self) -> DownloadProgressFoundHashSeq {
        match self {
            DownloadProgress::FoundHashSeq(f) => f.clone(),
            _ => panic!("DownloadProgress type is not 'FoundHashSeq'"),
        }
    }

    /// Return the `DownloadProgressProgress` event
    pub fn as_progress(&self) -> DownloadProgressProgress {
        match self {
            DownloadProgress::Progress(p) => p.clone(),
            _ => panic!("DownloadProgress type is not 'Progress'"),
        }
    }

    /// Return the `DownloadProgressDone` event
    pub fn as_done(&self) -> DownloadProgressDone {
        match self {
            DownloadProgress::Done(d) => d.clone(),
            _ => panic!("DownloadProgress type is not 'Done'"),
        }
    }

    /// Return the `DownloadProgressAllDone` event
    pub fn as_all_done(&self) -> DownloadProgressAllDone {
        match self {
            DownloadProgress::AllDone(e) => e.clone(),
            _ => panic!("DownloadProgress type is not 'AllDone'"),
        }
    }

    /// Return the `DownloadProgressAbort` event
    pub fn as_abort(&self) -> DownloadProgressAbort {
        match self {
            DownloadProgress::Abort(a) => a.clone(),
            _ => panic!("DownloadProgress type is not 'Abort'"),
        }
    }
}

/// A chunk range specification as a sequence of chunk offsets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Object)]
pub struct RangeSpec(pub(crate) iroh_blobs::protocol::RangeSpec);

#[uniffi::export]
impl RangeSpec {
    /// Checks if this [`RangeSpec`] does not select any chunks in the blob
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if this [`RangeSpec`] selects all chunks in the blob
    pub fn is_all(&self) -> bool {
        self.0.is_all()
    }
}

impl From<iroh_blobs::protocol::RangeSpec> for RangeSpec {
    fn from(h: iroh_blobs::protocol::RangeSpec) -> Self {
        RangeSpec(h)
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone, uniffi::Record)]
pub struct BlobInfo {
    /// Location of the blob
    pub path: String,
    /// The hash of the blob
    pub hash: Arc<Hash>,
    /// The size of the blob
    pub size: u64,
}

impl From<iroh_blobs::rpc::client::blobs::BlobInfo> for BlobInfo {
    fn from(value: iroh_blobs::rpc::client::blobs::BlobInfo) -> Self {
        BlobInfo {
            path: value.path,
            hash: Arc::new(value.hash.into()),
            size: value.size,
        }
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct IncompleteBlobInfo {
    /// The size we got
    pub size: u64,
    /// The size we expect
    pub expected_size: u64,
    /// The hash of the blob
    pub hash: Arc<Hash>,
}

impl From<iroh_blobs::rpc::client::blobs::IncompleteBlobInfo> for IncompleteBlobInfo {
    fn from(value: iroh_blobs::rpc::client::blobs::IncompleteBlobInfo) -> Self {
        IncompleteBlobInfo {
            size: value.size,
            expected_size: value.expected_size,
            hash: Arc::new(value.hash.into()),
        }
    }
}

/// A response to a list collections request
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct CollectionInfo {
    /// Tag of the collection
    pub tag: Vec<u8>,
    /// Hash of the collection
    pub hash: Arc<Hash>,
    /// Number of children in the collection
    ///
    /// This is an optional field, because the data is not always available.
    pub total_blobs_count: Option<u64>,
    /// Total size of the raw data referred to by all links
    ///
    /// This is an optional field, because the data is not always available.
    pub total_blobs_size: Option<u64>,
}

impl From<iroh_blobs::rpc::client::blobs::CollectionInfo> for CollectionInfo {
    fn from(value: iroh_blobs::rpc::client::blobs::CollectionInfo) -> Self {
        CollectionInfo {
            tag: value.tag.0.to_vec(),
            hash: Arc::new(value.hash.into()),
            total_blobs_count: value.total_blobs_count,
            total_blobs_size: value.total_blobs_size,
        }
    }
}

/// A collection of blobs
#[derive(Debug, uniffi::Object)]
pub struct Collection(pub(crate) RwLock<iroh_blobs::format::collection::Collection>);

impl From<iroh_blobs::format::collection::Collection> for Collection {
    fn from(value: iroh_blobs::format::collection::Collection) -> Self {
        Collection(RwLock::new(value))
    }
}

impl From<Collection> for iroh_blobs::format::collection::Collection {
    fn from(value: Collection) -> Self {
        let col = value.0.read().expect("Collection lock poisoned");
        col.clone()
    }
}

#[uniffi::export]
impl Collection {
    /// Create a new empty collection
    #[allow(clippy::new_without_default)]
    #[uniffi::constructor]
    pub fn new() -> Self {
        Collection(RwLock::new(
            iroh_blobs::format::collection::Collection::default(),
        ))
    }

    /// Add the given blob to the collection
    pub fn push(&self, name: String, hash: &Hash) -> Result<(), IrohError> {
        self.0.write().unwrap().push(name, hash.0);
        Ok(())
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> Result<bool, IrohError> {
        Ok(self.0.read().unwrap().is_empty())
    }

    /// Get the names of the blobs in this collection
    pub fn names(&self) -> Result<Vec<String>, IrohError> {
        Ok(self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(name, _)| name.clone())
            .collect())
    }

    /// Get the links to the blobs in this collection
    pub fn links(&self) -> Result<Vec<Arc<Hash>>, IrohError> {
        Ok(self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(_, hash)| Arc::new(Hash(*hash)))
            .collect())
    }

    /// Get the blobs associated with this collection
    pub fn blobs(&self) -> Result<Vec<LinkAndName>, IrohError> {
        Ok(self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(name, hash)| LinkAndName {
                name: name.clone(),
                link: Arc::new(Hash(*hash)),
            })
            .collect())
    }

    /// Returns the number of blobs in this collection
    pub fn len(&self) -> Result<u64, IrohError> {
        Ok(self.0.read().unwrap().len() as _)
    }
}

/// `LinkAndName` includes a name and a hash for a blob in a collection
#[derive(Clone, Debug, uniffi::Record)]
pub struct LinkAndName {
    /// The name associated with this [`Hash`]
    pub name: String,
    /// The [`Hash`] of the blob
    pub link: Arc<Hash>,
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::node::Iroh;
    use crate::{setup_logging, CallbackError, NodeOptions};

    use rand::RngCore;

    #[test]
    fn test_hash() {
        let hash_str = "6vp273v6cqbbq7xesa2xfrdt3oajykgeifprn3pj4p6y76654amq";
        let hex_str = "f55fafeebe1402187ee4903572c473db809c28c4415f16ede9e3fd8ffbdde019";
        let bytes = b"\xf5\x5f\xaf\xee\xbe\x14\x02\x18\x7e\xe4\x90\x35\x72\xc4\x73\xdb\x80\x9c\x28\xc4\x41\x5f\x16\xed\xe9\xe3\xfd\x8f\xfb\xdd\xe0\x19".to_vec();

        // create hash from string
        let hash = Hash::from_string(hash_str.into()).unwrap();

        // test methods are as expected
        assert_eq!(hash_str.to_string(), hash.to_string());
        assert_eq!(bytes.to_vec(), hash.to_bytes());
        assert_eq!(hex_str.to_string(), hash.to_hex());

        // create hash from bytes
        let hash_0 = Hash::from_bytes(bytes.clone()).unwrap();

        // test methods are as expected
        assert_eq!(hash_str.to_string(), hash_0.to_string());
        assert_eq!(bytes, hash_0.to_bytes());
        assert_eq!(hex_str.to_string(), hash_0.to_hex());

        // test that the eq function works
        assert!(hash.equal(&hash_0));
        assert!(hash_0.equal(&hash));
    }

    #[tokio::test]
    async fn test_blobs_add_get_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let node = Iroh::persistent(dir.keep().display().to_string())
            .await
            .unwrap();

        let sizes = [1, 10, 100, 1000, 10000, 100000];
        let mut hashes = Vec::new();
        for size in sizes.iter() {
            let hash = blobs_add_get_bytes_size(&node, *size);
            hashes.push(hash)
        }
    }

    async fn blobs_add_get_bytes_size(node: &Iroh, size: usize) -> Arc<Hash> {
        // create bytes
        let mut bytes = vec![0; size];
        rand::thread_rng().fill_bytes(&mut bytes);
        // add blob
        let add_outcome = node.blobs().add_bytes(bytes.to_vec()).await.unwrap();
        // check outcome
        assert_eq!(add_outcome.format, BlobFormat::Raw);
        assert_eq!(add_outcome.size, size as u64);
        // check size
        let hash = add_outcome.hash;
        let got_size = node.blobs().size(&hash).await.unwrap();
        assert_eq!(got_size, size as u64);
        //
        // get blob
        let got_bytes = node.blobs().read_to_bytes(hash.clone()).await.unwrap();
        assert_eq!(got_bytes.len(), size);
        assert_eq!(got_bytes, bytes);
        hash
    }

    #[tokio::test]
    async fn test_blob_read_write_path() {
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = Iroh::persistent(iroh_dir.keep().display().to_string())
            .await
            .unwrap();

        // create bytes
        let blob_size = 100;
        let mut bytes = vec![0; blob_size];
        rand::thread_rng().fill_bytes(&mut bytes);

        // write to file
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("in");
        let mut file = std::fs::File::create(path.clone()).unwrap();
        file.write_all(&bytes).unwrap();

        // add blob
        let tag = SetTagOption::auto();
        let wrap = WrapOption::no_wrap();

        struct Output {
            hash: Option<Arc<Hash>>,
            format: Option<BlobFormat>,
        }
        let output = Arc::new(Mutex::new(Output {
            hash: None,
            format: None,
        }));
        struct Callback {
            output: Arc<Mutex<Output>>,
        }

        #[async_trait::async_trait]
        impl AddCallback for Callback {
            async fn progress(&self, progress: Arc<AddProgress>) -> Result<(), CallbackError> {
                match *progress {
                    AddProgress::AllDone(ref d) => {
                        let mut output = self.output.lock().unwrap();
                        output.hash = Some(d.hash.clone());
                        output.format = Some(d.format.clone());
                    }
                    AddProgress::Abort(ref _a) => {
                        // anyhow::anyhow!("{}", a.error).into());
                        return Err(CallbackError::Error);
                    }
                    _ => {}
                }
                Ok(())
            }
        }
        let cb = Callback {
            output: output.clone(),
        };

        node.blobs()
            .add_from_path(
                path.display().to_string(),
                false,
                Arc::new(tag),
                Arc::new(wrap),
                Arc::new(cb),
            )
            .await
            .unwrap();

        let (hash, format) = {
            let output = output.lock().unwrap();
            let hash = output.hash.as_ref().cloned().unwrap();
            let format = output.format.as_ref().cloned().unwrap();
            (hash, format)
        };

        // check outcome info is as expected
        assert_eq!(BlobFormat::Raw, format);

        // check we get the expected size from the hash
        let got_size = node.blobs().size(&hash).await.unwrap();
        assert_eq!(blob_size as u64, got_size);

        // get bytes
        let got_bytes = node.blobs().read_to_bytes(hash.clone()).await.unwrap();
        assert_eq!(blob_size, got_bytes.len());
        assert_eq!(bytes, got_bytes);

        // write to file
        let out_path = dir.path().join("out");
        node.blobs()
            .write_to_path(hash, out_path.display().to_string())
            .await
            .unwrap();

        // open file
        let got_bytes = std::fs::read(out_path).unwrap();
        assert_eq!(blob_size, got_bytes.len());
        assert_eq!(bytes, got_bytes);
    }

    #[tokio::test]
    async fn test_blobs_list_collections() {
        let dir = tempfile::tempdir().unwrap();
        let num_blobs = 3;
        let blob_size = 100;
        for i in 0..num_blobs {
            let path = dir.path().join(i.to_string());
            let mut file = std::fs::File::create(path).unwrap();
            let mut bytes = vec![0; blob_size];
            rand::thread_rng().fill_bytes(&mut bytes);
            file.write_all(&bytes).unwrap()
        }

        let iroh_dir = tempfile::tempdir().unwrap();
        let node = Iroh::persistent(iroh_dir.keep().display().to_string())
            .await
            .unwrap();

        // ensure there are no blobs to start
        let blobs = node.blobs().list().await.unwrap();
        assert!(blobs.is_empty());

        struct Output {
            collection_hash: Option<Arc<Hash>>,
            format: Option<BlobFormat>,
            blob_hashes: Vec<Arc<Hash>>,
        }
        let output = Arc::new(Mutex::new(Output {
            collection_hash: None,
            format: None,
            blob_hashes: Vec::new(),
        }));
        struct Callback {
            output: Arc<Mutex<Output>>,
        }

        #[async_trait::async_trait]
        impl AddCallback for Callback {
            async fn progress(&self, progress: Arc<AddProgress>) -> Result<(), CallbackError> {
                match *progress {
                    AddProgress::AllDone(ref d) => {
                        let mut output = self.output.lock().unwrap();
                        output.collection_hash = Some(d.hash.clone());
                        output.format = Some(d.format.clone());
                    }
                    AddProgress::Abort(ref _a) => {
                        return Err(CallbackError::Error);
                        // return Err(anyhow::anyhow!("{}", a.error).into());
                    }
                    AddProgress::Done(ref d) => {
                        let mut output = self.output.lock().unwrap();
                        output.blob_hashes.push(d.hash.clone())
                    }
                    _ => {}
                }
                Ok(())
            }
        }

        let cb = Callback {
            output: output.clone(),
        };

        node.blobs()
            .add_from_path(
                dir.keep().display().to_string(),
                false,
                Arc::new(SetTagOption::Auto),
                Arc::new(WrapOption::NoWrap),
                Arc::new(cb),
            )
            .await
            .unwrap();

        let collections = node.blobs().list_collections().await.unwrap();
        assert!(collections.len() == 1);
        let (collection_hash, blob_hashes) = {
            let output = output.lock().unwrap();
            let collection_hash = output.collection_hash.as_ref().cloned().unwrap();
            let mut blob_hashes = output.blob_hashes.clone();
            blob_hashes.push(collection_hash.clone());
            (collection_hash, blob_hashes)
        };
        assert_eq!(*(collections[0].hash), *collection_hash);
        assert_eq!(
            collections[0].total_blobs_count.unwrap(),
            blob_hashes.len() as u64
        );

        let blobs = node.blobs().list().await.unwrap();
        hashes_exist(&blob_hashes, &blobs);
        println!("finished");
    }

    fn hashes_exist(expect: &Vec<Arc<Hash>>, got: &[Arc<Hash>]) {
        for hash in expect {
            if !got.contains(hash) {
                panic!("expected to find hash {} in the list of hashes", hash);
            }
        }
    }

    #[tokio::test]
    async fn test_list_and_delete() {
        setup_logging();

        let iroh_dir = tempfile::tempdir().unwrap();
        // we're going to use a very fast GC interval to get this test to delete stuff aggressively
        let opts = NodeOptions {
            gc_interval_millis: Some(50),
            ..Default::default()
        };
        let node = Iroh::persistent_with_options(iroh_dir.keep().display().to_string(), opts)
            .await
            .unwrap();

        // create bytes
        let blob_size = 100;
        let mut blobs = vec![];
        let num_blobs = 3;

        for _i in 0..num_blobs {
            let mut bytes = vec![0; blob_size];
            rand::thread_rng().fill_bytes(&mut bytes);
            blobs.push(bytes);
        }

        let mut hashes = vec![];
        let mut tags = vec![];
        for blob in blobs {
            let output = node.blobs().add_bytes(blob).await.unwrap();
            hashes.push(output.hash);
            tags.push(output.tag);
        }

        let got_hashes = node.blobs().list().await.unwrap();
        assert_eq!(num_blobs, got_hashes.len());
        hashes_exist(&hashes, &got_hashes);

        for hash in &got_hashes {
            assert!(node.blobs().has(hash).await.unwrap());
        }

        let remove_hash = hashes.pop().unwrap();
        let remove_tag = tags.pop().unwrap();
        // delete the tag for the first blob
        node.tags().delete(remove_tag).await.unwrap();
        // wait for GC to clear the blob. windows test runner is slow & needs like 500ms
        tokio::time::sleep(Duration::from_secs(1)).await;

        let got_hashes = node.blobs().list().await.unwrap();
        assert_eq!(num_blobs - 1, got_hashes.len());
        hashes_exist(&hashes, &got_hashes);

        for hash in got_hashes {
            if remove_hash.equal(&hash) {
                panic!("blob {} should have been removed", remove_hash);
            }
        }

        assert!(!node.blobs().has(&remove_hash).await.unwrap());
    }
}
