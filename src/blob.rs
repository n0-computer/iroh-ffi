use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};

use futures::{StreamExt, TryStreamExt};

use crate::node::IrohNode;
use crate::{block_on, IrohError, NodeAddr, Tag};

impl IrohNode {
    /// List all complete blobs.
    ///
    /// Note: this allocates for each `BlobListResponse`, if you have many `BlobListReponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn blobs_list(&self) -> Result<Vec<Arc<Hash>>, IrohError> {
        block_on(&self.async_runtime, async {
            let response = self
                .sync_client
                .blobs
                .list()
                .await
                .map_err(IrohError::blobs)?;

            let hashes: Vec<Arc<Hash>> = response
                .map_ok(|i| Arc::new(Hash(i.hash)))
                .map_err(IrohError::blobs)
                .try_collect()
                .await?;

            Ok(hashes)
        })
    }

    /// Get the size information on a single blob.
    ///
    /// Method only exist in FFI
    pub fn blobs_size(&self, hash: Arc<Hash>) -> Result<u64, IrohError> {
        block_on(&self.async_runtime, async {
            let r = self
                .sync_client
                .blobs
                .read(hash.0)
                .await
                .map_err(IrohError::blobs)?;
            Ok(r.size())
        })
    }

    /// Read all bytes of single blob.
    ///
    /// This allocates a buffer for the full blob. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_to_bytes`].
    pub fn blobs_read_to_bytes(&self, hash: Arc<Hash>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .blobs
                .read_to_bytes(hash.0)
                .await
                .map(|b| b.to_vec())
                .map_err(IrohError::blobs)
        })
    }

    /// Import a blob from a filesystem path.
    ///
    /// `path` should be an absolute path valid for the file system on which
    /// the node runs.
    /// If `in_place` is true, Iroh will assume that the data will not change and will share it in
    /// place without copying to the Iroh data directory.
    pub fn blobs_add_from_path(
        &self,
        path: String,
        in_place: bool,
        tag: Arc<SetTagOption>,
        wrap: Arc<WrapOption>,
        cb: Box<dyn AddCallback>,
    ) -> Result<(), IrohError> {
        block_on(&self.async_runtime, async {
            let mut stream = self
                .sync_client
                .blobs
                .add_from_path(
                    path.into(),
                    in_place,
                    (*tag).clone().into(),
                    (*wrap).clone().into(),
                )
                .await
                .map_err(IrohError::blobs)?;
            while let Some(progress) = stream.next().await {
                let progress = progress.map_err(IrohError::blobs)?;
                if let Err(e) = cb.progress(Arc::new(progress.into())) {
                    return Err(e);
                }
            }
            Ok(())
        })
    }

    /// Export the blob contents to a file path
    /// The `path` field is expected to be the absolute path.
    pub fn blobs_write_to_path(&self, hash: Arc<Hash>, path: String) -> Result<(), IrohError> {
        block_on(&self.async_runtime, async {
            let mut reader = self
                .sync_client
                .blobs
                .read(hash.0)
                .await
                .map_err(IrohError::blobs)?;
            let path: PathBuf = path.into();
            if let Some(dir) = path.parent() {
                tokio::fs::create_dir_all(dir)
                    .await
                    .map_err(IrohError::blobs)?;
            }
            let mut file = tokio::fs::File::create(path)
                .await
                .map_err(IrohError::blobs)?;
            tokio::io::copy(&mut reader, &mut file)
                .await
                .map_err(IrohError::blobs)?;
            Ok(())
        })
    }

    /// Write a blob by passing bytes.
    pub fn blobs_add_bytes(
        &self,
        bytes: Vec<u8>,
        tag: Arc<SetTagOption>,
    ) -> Result<BlobAddOutcome, IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .blobs
                .add_bytes(bytes.into(), (*tag).clone().into())
                .await
                .map(|outcome| outcome.into())
                .map_err(IrohError::blobs)
        })
    }

    /// Download a blob from another node and add it to the local database.
    pub fn blobs_download(
        &self,
        req: Arc<BlobDownloadRequest>,
        cb: Box<dyn DownloadCallback>,
    ) -> Result<(), IrohError> {
        block_on(&self.async_runtime, async {
            let mut stream = self
                .sync_client
                .blobs
                .download(req.0.clone())
                .await
                .map_err(IrohError::blobs)?;
            while let Some(progress) = stream.next().await {
                let progress = progress.map_err(IrohError::blobs)?;
                if let Err(e) = cb.progress(Arc::new(progress.into())) {
                    return Err(e);
                }
            }
            Ok(())
        })
    }

    /// List all incomplete (partial) blobs.
    ///
    /// Note: this allocates for each `BlobListIncompleteResponse`, if you have many `BlobListIncompleteResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn blobs_list_incomplete(&self) -> Result<Vec<BlobListIncompleteResponse>, IrohError> {
        block_on(&self.async_runtime, async {
            let blobs = self
                .sync_client
                .blobs
                .list_incomplete()
                .await
                .map_err(IrohError::blobs)?
                .map_ok(|res| res.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::blobs)?;
            Ok(blobs)
        })
    }

    /// List all collections.
    ///
    /// Note: this allocates for each `BlobListCollectionsResponse`, if you have many `BlobListCollectionsResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn blobs_list_collections(&self) -> Result<Vec<BlobListCollectionsResponse>, IrohError> {
        block_on(&self.async_runtime, async {
            let blobs = self
                .sync_client
                .blobs
                .list_collections()
                .await
                .map_err(IrohError::blobs)?
                .map_ok(|res| res.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::blobs)?;
            Ok(blobs)
        })
    }

    /// Delete a blob.
    pub fn blobs_delete_blob(&self, hash: Arc<Hash>) -> Result<(), IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .blobs
                .delete_blob((*hash).clone().0)
                .await
                .map_err(IrohError::blobs)
        })
    }
}

/// Outcome of a blob add operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobAddOutcome {
    /// The hash of the blob
    pub hash: Arc<Hash>,
    /// The format the blob
    pub format: BlobFormat,
    /// The size of the blob
    pub size: u64,
    /// The tag of the blob
    pub tag: Arc<Tag>,
}

impl From<iroh::client::BlobAddOutcome> for BlobAddOutcome {
    fn from(value: iroh::client::BlobAddOutcome) -> Self {
        BlobAddOutcome {
            hash: Arc::new(value.hash.into()),
            format: value.format.into(),
            size: value.size,
            tag: Arc::new(value.tag.into()),
        }
    }
}

/// An option for commands that allow setting a Tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetTagOption {
    /// A tag will be automatically generated
    Auto,
    /// The tag is explicitly named
    Named(Arc<Tag>),
}

impl SetTagOption {
    /// Indicate you want an automatically generated tag
    pub fn auto() -> Self {
        SetTagOption::Auto
    }

    /// Indicate you want a named tag
    pub fn named(tag: Arc<Tag>) -> Self {
        SetTagOption::Named(tag)
    }
}

impl From<SetTagOption> for iroh::rpc_protocol::SetTagOption {
    fn from(value: SetTagOption) -> Self {
        match value {
            SetTagOption::Auto => iroh::rpc_protocol::SetTagOption::Auto,
            SetTagOption::Named(tag) => iroh::rpc_protocol::SetTagOption::Named(tag.0.clone()),
        }
    }
}

/// Whether to wrap the added data in a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrapOption {
    /// Do not wrap the file or directory.
    NoWrap,
    /// Wrap the file or directory in a colletion.
    Wrap {
        /// Override the filename in the wrapping collection.
        name: Option<String>,
    },
}

impl WrapOption {
    /// Indicate you do not wrap the file or directory.
    pub fn no_wrap() -> Self {
        WrapOption::NoWrap
    }

    /// Indicate you want to wrap the file or directory in a colletion, with an optional name
    pub fn wrap(name: Option<String>) -> Self {
        WrapOption::Wrap { name }
    }
}

impl From<WrapOption> for iroh::rpc_protocol::WrapOption {
    fn from(value: WrapOption) -> Self {
        match value {
            WrapOption::NoWrap => iroh::rpc_protocol::WrapOption::NoWrap,
            WrapOption::Wrap { name } => iroh::rpc_protocol::WrapOption::Wrap { name },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash(pub(crate) iroh::bytes::Hash);

impl From<iroh::bytes::Hash> for Hash {
    fn from(h: iroh::bytes::Hash) -> Self {
        Hash(h)
    }
}

impl Hash {
    /// Calculate the hash of the provide bytes.
    pub fn new(buf: Vec<u8>) -> Self {
        Hash(iroh::bytes::Hash::new(buf))
    }

    /// Bytes of the hash.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Create a `Hash` from its raw bytes representation.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        let bytes: [u8; 32] = bytes.try_into().map_err(|b: Vec<u8>| {
            IrohError::hash(format!("expected byte array of length 32, got {}", b.len()))
        })?;
        Ok(Hash(iroh::bytes::Hash::from_bytes(bytes)))
    }

    /// Make a Hash from hex or base 64 encoded cid string
    pub fn from_string(s: String) -> Result<Self, IrohError> {
        match iroh::bytes::Hash::from_str(&s) {
            Ok(key) => Ok(key.into()),
            Err(err) => Err(IrohError::hash(err)),
        }
    }

    /// Get the cid as bytes.
    pub fn as_cid_bytes(&self) -> Vec<u8> {
        self.0.as_cid_bytes().to_vec()
    }

    /// Try to create a blake3 cid from cid bytes.
    ///
    /// This will only work if the prefix is the following:
    /// - version 1
    /// - raw codec
    /// - blake3 hash function
    /// - 32 byte hash size
    pub fn from_cid_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        Ok(Hash(
            iroh::bytes::Hash::from_cid_bytes(&bytes).map_err(IrohError::hash)?,
        ))
    }

    /// Convert the hash to a hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns true if the Hash's have the same value
    pub fn equal(&self, other: Arc<Hash>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Hash> for iroh::bytes::Hash {
    fn from(value: Hash) -> Self {
        value.0
    }
}

pub trait AddCallback: Send + Sync + 'static {
    fn progress(&self, progress: Arc<AddProgress>) -> Result<(), IrohError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddProgressType {
    Found,
    Progress,
    Done,
    AllDone,
    Abort,
}

/// An AddProgress event indicating an item was found with name `name`, that can be referred to by `id`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// An AddProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// An AddProgress event indicated we are done with `id` and now have a hash `hash`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProgressDone {
    /// The unique id of the entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// An AddProgress event indicating we are done with the the whole operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProgressAllDone {
    /// The hash of the created data.
    pub hash: Arc<Hash>,
    /// The format of the added data.
    pub format: BlobFormat,
    /// The tag of the added data.
    pub tag: Arc<Tag>,
}

/// An AddProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProgressAbort {
    pub error: String,
}

/// Progress updates for the add operation.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl From<iroh::bytes::provider::AddProgress> for AddProgress {
    fn from(value: iroh::bytes::provider::AddProgress) -> Self {
        match value {
            iroh::bytes::provider::AddProgress::Found { id, name, size } => {
                AddProgress::Found(AddProgressFound { id, name, size })
            }
            iroh::bytes::provider::AddProgress::Progress { id, offset } => {
                AddProgress::Progress(AddProgressProgress { id, offset })
            }
            iroh::bytes::provider::AddProgress::Done { id, hash } => {
                AddProgress::Done(AddProgressDone {
                    id,
                    hash: Arc::new(hash.into()),
                })
            }
            iroh::bytes::provider::AddProgress::AllDone { hash, format, tag } => {
                AddProgress::AllDone(AddProgressAllDone {
                    hash: Arc::new(hash.into()),
                    format: format.into(),
                    tag: Arc::new(tag.into()),
                })
            }
            iroh::bytes::provider::AddProgress::Abort(err) => {
                AddProgress::Abort(AddProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobFormat {
    /// Raw blob
    Raw,
    /// A sequence of BLAKE3 hashes
    HashSeq,
}

impl From<iroh::rpc_protocol::BlobFormat> for BlobFormat {
    fn from(value: iroh::rpc_protocol::BlobFormat) -> Self {
        match value {
            iroh::rpc_protocol::BlobFormat::Raw => BlobFormat::Raw,
            iroh::rpc_protocol::BlobFormat::HashSeq => BlobFormat::HashSeq,
        }
    }
}

impl From<BlobFormat> for iroh::rpc_protocol::BlobFormat {
    fn from(value: BlobFormat) -> Self {
        match value {
            BlobFormat::Raw => iroh::rpc_protocol::BlobFormat::Raw,
            BlobFormat::HashSeq => iroh::rpc_protocol::BlobFormat::HashSeq,
        }
    }
}

/// A Request token is an opaque byte sequence associated with a single request.
/// Applications can use request tokens to implement request authorization,
/// user association, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestToken(iroh::bytes::protocol::RequestToken);

impl RequestToken {
    /// Creates a new request token from bytes.
    pub fn new(bytes: Vec<u8>) -> Result<Self, IrohError> {
        Ok(RequestToken(
            iroh::bytes::protocol::RequestToken::new(bytes).map_err(IrohError::request_token)?,
        ))
    }

    /// Generate a random 32 byte request token.
    pub fn generate() -> Self {
        RequestToken(iroh::bytes::protocol::RequestToken::generate())
    }

    /// Returns a reference the token bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Create a request token from a string
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        Ok(RequestToken(
            iroh::bytes::protocol::RequestToken::from_str(&str)
                .map_err(IrohError::request_token)?,
        ))
    }

    /// Returns true if both RequestTokens have the same value
    pub fn equal(&self, other: Arc<RequestToken>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for RequestToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Location to store a downloaded blob at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadLocation {
    /// Store in the node's blob storage directory.
    Internal,
    /// Store at the provided path.
    External {
        /// The path to store the data at.
        path: String,
        /// If this flag is true, the data is shared in place, i.e. it is moved to the
        /// out path instead of being copied. The database itself contains only a
        /// reference to the out path of the file.
        ///
        /// If the data is modified in the location specified by the out path,
        /// download attempts for the associated hash will fail.
        in_place: bool,
    },
}

impl DownloadLocation {
    /// Store in the node's blob storage directory.
    pub fn internal() -> Self {
        DownloadLocation::Internal
    }

    /// Store at the provided path.
    ///
    /// If `in_place` is true, the data is shared in place, i.e. it is moved to the
    /// out path instead of being copied. The database itself contains only a
    /// reference to the out path of the file.
    ///
    /// If the data is modified in the location specified by the out path,
    /// download attempts for the associated hash will fail.
    pub fn external(path: String, in_place: bool) -> Self {
        DownloadLocation::External { in_place, path }
    }
}

impl From<DownloadLocation> for iroh::rpc_protocol::DownloadLocation {
    fn from(value: DownloadLocation) -> Self {
        match value {
            DownloadLocation::Internal => iroh::rpc_protocol::DownloadLocation::Internal,
            DownloadLocation::External { path, in_place } => {
                iroh::rpc_protocol::DownloadLocation::External { path, in_place }
            }
        }
    }
}

/// A request to the node to download and share the data specified by the hash.
pub struct BlobDownloadRequest(iroh::rpc_protocol::BlobDownloadRequest);
impl BlobDownloadRequest {
    /// Create a BlobDownloadRequest
    pub fn new(
        hash: Arc<Hash>,
        format: BlobFormat,
        node: Arc<NodeAddr>,
        tag: Arc<SetTagOption>,
        out: Arc<DownloadLocation>,
        token: Option<Arc<RequestToken>>,
    ) -> Self {
        BlobDownloadRequest(iroh::rpc_protocol::BlobDownloadRequest {
            hash: (*hash).0.clone(),
            format: format.into(),
            peer: (*node).clone().into(),
            token: token.map(|token| (*token).clone().0),
            tag: (*tag).clone().into(),
            out: (*out).clone().into(),
        })
    }
}

pub trait DownloadCallback: Send + Sync + 'static {
    fn progress(&self, progress: Arc<DownloadProgress>) -> Result<(), IrohError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadProgressType {
    Connected,
    Found,
    FoundHashSeq,
    Progress,
    Done,
    NetworkDone,
    Export,
    ExportProgress,
    AllDone,
    Abort,
}

/// A DownloadProgress event indicating an item was found with hash `hash`, that can be referred to by `id`
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// A DownloadProgress event indicating an item was found with hash `hash`, that can be referred to by `id`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressFoundHashSeq {
    /// Number of children in the collection, if known.
    pub children: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// A DownloadProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DownloadProgress event indicated we are done with `id`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressDone {
    /// The unique id of the entry.
    pub id: u64,
}

/// A DownloadProgress event indicating we are done with the networking portion - all data is local
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressNetworkDone {
    /// The number of bytes written
    pub bytes_written: u64,
    /// The number of bytes read
    pub bytes_read: u64,
    /// The time it took to transfer the data
    pub elapsed: Duration,
}

/// A DownloadProgress event indicating the download part is done for this id, we are not exporting
/// the data to the specified path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressExport {
    /// Unique id of the entry
    pub id: u64,
    /// The hash of the entry
    pub hash: Arc<Hash>,
    /// The size of the entry in bytes
    pub size: u64,
    /// The path to the file where the data is exported
    pub target: String,
}

/// A DownloadProgress event indicating We have made progress exporting the data.
///
/// This is only sent for large blobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressExportProgress {
    /// Unique id of the entry that is being exported.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DownloadProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgressAbort {
    pub error: String,
}

/// Progress updates for the add operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadProgress {
    /// A new connection was established.
    Connected,
    /// An item was found with hash `hash`, from now on referred to via `id`
    Found(DownloadProgressFound),
    /// An item was found with hash `hash`, from now on referred to via `id`
    FoundHashSeq(DownloadProgressFoundHashSeq),
    /// We got progress ingesting item `id`.
    Progress(DownloadProgressProgress),
    /// We are done with `id`, and the hash is `hash`.
    Done(DownloadProgressDone),
    /// We are done with the network part - all data is local
    NetworkDone(DownloadProgressNetworkDone),
    /// The download part is done for this id, we are not exporting the data to the specified out
    /// path
    Export(DownloadProgressExport),
    /// We have made progress exporting the data
    ///
    /// This is only sent for large blobs
    ExportProgress(DownloadProgressExportProgress),
    /// We are done with the whole operation.
    AllDone,
    /// We got an error and need to abort.
    ///
    /// This will be the last message in the stream.
    Abort(DownloadProgressAbort),
}

impl From<iroh::bytes::provider::DownloadProgress> for DownloadProgress {
    fn from(value: iroh::bytes::provider::DownloadProgress) -> Self {
        match value {
            iroh::bytes::provider::DownloadProgress::Connected => DownloadProgress::Connected,
            iroh::bytes::provider::DownloadProgress::Found {
                id,
                hash,
                child,
                size,
            } => DownloadProgress::Found(DownloadProgressFound {
                id,
                hash: Arc::new(hash.into()),
                child,
                size,
            }),
            iroh::bytes::provider::DownloadProgress::FoundHashSeq { hash, children } => {
                DownloadProgress::FoundHashSeq(DownloadProgressFoundHashSeq {
                    hash: Arc::new(hash.into()),
                    children,
                })
            }
            iroh::bytes::provider::DownloadProgress::Progress { id, offset } => {
                DownloadProgress::Progress(DownloadProgressProgress { id, offset })
            }
            iroh::bytes::provider::DownloadProgress::Done { id } => {
                DownloadProgress::Done(DownloadProgressDone { id })
            }
            iroh::bytes::provider::DownloadProgress::NetworkDone {
                bytes_written,
                bytes_read,
                elapsed,
            } => DownloadProgress::NetworkDone(DownloadProgressNetworkDone {
                bytes_written,
                bytes_read,
                elapsed,
            }),
            iroh::bytes::provider::DownloadProgress::Export {
                id,
                hash,
                size,
                target,
            } => DownloadProgress::Export(DownloadProgressExport {
                id,
                hash: Arc::new(hash.into()),
                size,
                target,
            }),
            iroh::bytes::provider::DownloadProgress::ExportProgress { id, offset } => {
                DownloadProgress::ExportProgress(DownloadProgressExportProgress { id, offset })
            }
            iroh::bytes::provider::DownloadProgress::AllDone => DownloadProgress::AllDone,
            iroh::bytes::provider::DownloadProgress::Abort(err) => {
                DownloadProgress::Abort(DownloadProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

impl DownloadProgress {
    /// Get the type of event
    pub fn r#type(&self) -> DownloadProgressType {
        match self {
            DownloadProgress::Connected => DownloadProgressType::Connected,
            DownloadProgress::Found(_) => DownloadProgressType::Found,
            DownloadProgress::FoundHashSeq(_) => DownloadProgressType::FoundHashSeq,
            DownloadProgress::Progress(_) => DownloadProgressType::Progress,
            DownloadProgress::Done(_) => DownloadProgressType::Done,
            DownloadProgress::NetworkDone(_) => DownloadProgressType::NetworkDone,
            DownloadProgress::Export(_) => DownloadProgressType::Export,
            DownloadProgress::ExportProgress(_) => DownloadProgressType::ExportProgress,
            DownloadProgress::AllDone => DownloadProgressType::AllDone,
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

    /// Return the `DownloadProgressNetworkDone` event
    pub fn as_network_done(&self) -> DownloadProgressNetworkDone {
        match self {
            DownloadProgress::NetworkDone(n) => n.clone(),
            _ => panic!("DownloadProgress type is not 'NetworkDone'"),
        }
    }

    /// Return the `DownloadProgressExport` event
    pub fn as_export(&self) -> DownloadProgressExport {
        match self {
            DownloadProgress::Export(e) => e.clone(),
            _ => panic!("DownloadProgress type is not 'Export'"),
        }
    }

    /// Return the `DownloadProgressExportProgress` event
    pub fn as_export_progress(&self) -> DownloadProgressExportProgress {
        match self {
            DownloadProgress::ExportProgress(e) => e.clone(),
            _ => panic!("DownloadProgress type is not 'ExportProgress'"),
        }
    }

    /// Return the `DownloadProgressAbort`
    pub fn as_abort(&self) -> DownloadProgressAbort {
        match self {
            DownloadProgress::Abort(a) => a.clone(),
            _ => panic!("DownloadProgress type is not 'Abort'"),
        }
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone)]
pub struct BlobListResponse {
    /// Location of the blob
    pub path: String,
    /// The hash of the blob
    pub hash: Arc<Hash>,
    /// The size of the blob
    pub size: u64,
}

impl From<iroh::rpc_protocol::BlobListResponse> for BlobListResponse {
    fn from(value: iroh::rpc_protocol::BlobListResponse) -> Self {
        BlobListResponse {
            path: value.path,
            hash: Arc::new(value.hash.into()),
            size: value.size,
        }
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone)]
pub struct BlobListIncompleteResponse {
    /// The size we got
    pub size: u64,
    /// The size we expect
    pub expected_size: u64,
    /// The hash of the blob
    pub hash: Arc<Hash>,
}

impl From<iroh::rpc_protocol::BlobListIncompleteResponse> for BlobListIncompleteResponse {
    fn from(value: iroh::rpc_protocol::BlobListIncompleteResponse) -> Self {
        BlobListIncompleteResponse {
            size: value.size,
            expected_size: value.expected_size,
            hash: Arc::new(value.hash.into()),
        }
    }
}

/// A response to a list collections request
#[derive(Debug, Clone)]
pub struct BlobListCollectionsResponse {
    /// Tag of the collection
    pub tag: Arc<Tag>,
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

impl From<iroh::rpc_protocol::BlobListCollectionsResponse> for BlobListCollectionsResponse {
    fn from(value: iroh::rpc_protocol::BlobListCollectionsResponse) -> Self {
        BlobListCollectionsResponse {
            tag: Arc::new(value.tag.into()),
            hash: Arc::new(value.hash.into()),
            total_blobs_count: value.total_blobs_count,
            total_blobs_size: value.total_blobs_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::node::IrohNode;
    use rand::RngCore;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn test_hash() {
        let hash_str = "bafkr4ih6qxpyfyrgxbcrvmiqbm7hb5fdpn4yezj7ayh6gwto4hm2573glu";
        let hex_str = "fe85df82e226b8451ab1100b3e70f4a37b7982653f060fe35a6ee1d9aeff665d";
        let bytes = b"\xfe\x85\xdf\x82\xe2\x26\xb8\x45\x1a\xb1\x10\x0b\x3e\x70\xf4\xa3\x7b\x79\x82\x65\x3f\x06\x0f\xe3\x5a\x6e\xe1\xd9\xae\xff\x66\x5d".to_vec();
        let cid_prefix = b"\x01\x55\x1e\x20".to_vec();

        let cid_bytes = {
            let mut b = cid_prefix.clone();
            b.append(&mut bytes.clone());
            b
        };
        // create hash from string
        let hash = Hash::from_string(hash_str.into()).unwrap();

        // test methods are as expected
        assert_eq!(hash_str.to_string(), hash.to_string());
        assert_eq!(bytes.to_vec(), hash.to_bytes());
        assert_eq!(hex_str.to_string(), hash.to_hex());
        assert_eq!(cid_bytes, hash.as_cid_bytes());

        // create hash from bytes
        let hash_0 = Hash::from_bytes(bytes.clone()).unwrap();

        // test methods are as expected
        assert_eq!(hash_str.to_string(), hash_0.to_string());
        assert_eq!(bytes, hash_0.to_bytes());
        assert_eq!(hex_str.to_string(), hash_0.to_hex());
        assert_eq!(cid_bytes, hash_0.as_cid_bytes());

        // create hash from cid bytes
        let hash_1 = Hash::from_cid_bytes(cid_bytes.clone()).unwrap();

        // test methods are as expected
        assert_eq!(hash_str.to_string(), hash_1.to_string());
        assert_eq!(bytes, hash_1.to_bytes());
        assert_eq!(hex_str.to_string(), hash_1.to_hex());
        assert_eq!(cid_bytes, hash_1.as_cid_bytes());

        // test that the eq function works
        let hash = Arc::new(hash);
        let hash_0 = Arc::new(hash_0);
        let hash_1 = Arc::new(hash_1);
        assert!(hash.equal(hash_0.clone()));
        assert!(hash.equal(hash_1.clone()));
        assert!(hash_0.equal(hash.clone()));
        assert!(hash_0.equal(hash_1.clone()));
        assert!(hash_1.equal(hash));
        assert!(hash_1.equal(hash_0));
    }

    #[test]
    fn test_blobs_add_get_bytes() {
        // temp dir
        let dir = tempfile::tempdir().unwrap();
        let node = IrohNode::new(dir.into_path().display().to_string()).unwrap();
        let sizes = [1, 10, 100, 1000, 10000, 100000];
        let mut hashes = Vec::new();
        for size in sizes.iter() {
            let hash = blobs_add_get_bytes_size(&node, *size);
            hashes.push(hash)
        }
    }

    fn blobs_add_get_bytes_size(node: &IrohNode, size: usize) -> Arc<Hash> {
        // create bytes
        let mut bytes = vec![0; size];
        rand::thread_rng().fill_bytes(&mut bytes);
        // add blob
        let tag = SetTagOption::auto();
        let add_outcome = node.blobs_add_bytes(bytes.to_vec(), tag.into()).unwrap();
        // check outcome
        assert_eq!(add_outcome.format, BlobFormat::Raw);
        assert_eq!(add_outcome.size, size as u64);
        // check size
        let hash = add_outcome.hash;
        let got_size = node.blobs_size(hash.clone()).unwrap();
        assert_eq!(got_size, size as u64);
        //
        // get blob
        let got_bytes = node.blobs_read_to_bytes(hash.clone()).unwrap();
        assert_eq!(got_bytes.len(), size);
        assert_eq!(got_bytes, bytes);
        hash
    }

    #[test]
    fn test_blob_read_write_path() {
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = IrohNode::new(iroh_dir.into_path().display().to_string()).unwrap();

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

        impl AddCallback for Callback {
            fn progress(&self, progress: Arc<AddProgress>) -> Result<(), IrohError> {
                match *progress {
                    AddProgress::AllDone(ref d) => {
                        let mut output = self.output.lock().unwrap();
                        output.hash = Some(d.hash.clone());
                        output.format = Some(d.format.clone());
                    }
                    AddProgress::Abort(ref a) => {
                        return Err(IrohError::blobs(a.error.clone()));
                    }
                    _ => {}
                }
                Ok(())
            }
        }
        let cb = Callback {
            output: output.clone(),
        };

        node.blobs_add_from_path(
            path.display().to_string(),
            false,
            Arc::new(tag),
            Arc::new(wrap),
            Box::new(cb),
        )
        .unwrap();

        let (hash, format) = {
            let output = output.lock().unwrap();
            let hash = output.hash.as_ref().map(|h| h.clone()).unwrap();
            let format = output.format.as_ref().map(|h| h.clone()).unwrap();
            (hash, format)
        };

        // check outcome info is as expected
        assert_eq!(BlobFormat::Raw, format);

        // check we get the expected size from the hash
        let got_size = node.blobs_size(hash.clone()).unwrap();
        assert_eq!(blob_size as u64, got_size);

        // get bytes
        let got_bytes = node.blobs_read_to_bytes(hash.clone()).unwrap();
        assert_eq!(blob_size, got_bytes.len());
        assert_eq!(bytes, got_bytes);

        // write to file
        let out_path = dir.path().join("out");
        node.blobs_write_to_path(hash, out_path.display().to_string())
            .unwrap();

        // open file
        let got_bytes = std::fs::read(out_path).unwrap();
        assert_eq!(blob_size, got_bytes.len());
        assert_eq!(bytes, got_bytes);
    }

    #[test]
    fn test_blobs_list_collections() {
        // temp dir
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

        // temp dir
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = IrohNode::new(iroh_dir.into_path().display().to_string()).unwrap();

        // ensure there are no blobs to start
        let blobs = node.blobs_list().unwrap();
        assert!(blobs.len() == 0);

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

        impl AddCallback for Callback {
            fn progress(&self, progress: Arc<AddProgress>) -> Result<(), IrohError> {
                match *progress {
                    AddProgress::AllDone(ref d) => {
                        let mut output = self.output.lock().unwrap();
                        output.collection_hash = Some(d.hash.clone());
                        output.format = Some(d.format.clone());
                    }
                    AddProgress::Abort(ref a) => {
                        return Err(IrohError::blobs(a.error.clone()));
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

        node.blobs_add_from_path(
            dir.into_path().display().to_string(),
            false,
            Arc::new(SetTagOption::Auto),
            Arc::new(WrapOption::NoWrap),
            Box::new(cb),
        )
        .unwrap();

        let collections = node.blobs_list_collections().unwrap();
        assert!(collections.len() == 1);
        let (collection_hash, blob_hashes) = {
            let output = output.lock().unwrap();
            let collection_hash = output.collection_hash.as_ref().map(|h| h.clone()).unwrap();
            let mut blob_hashes = output.blob_hashes.clone();
            blob_hashes.push(collection_hash.clone());
            (collection_hash, blob_hashes)
        };
        assert_eq!(*(collections[0].hash), *collection_hash);
        assert_eq!(
            collections[0].total_blobs_count.unwrap(),
            blob_hashes.len() as u64
        );

        let blobs = node.blobs_list().unwrap();
        hashes_exist(&blob_hashes, &blobs);
        println!("finished");
    }

    fn hashes_exist(expect: &Vec<Arc<Hash>>, got: &Vec<Arc<Hash>>) {
        for hash in expect {
            if !got.contains(&hash) {
                panic!(
                    "expected to find hash {} in the list of hashes",
                    hash.to_string()
                );
            }
        }
    }

    #[test]
    fn test_list_and_delete() {
        // temp dir
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = IrohNode::new(iroh_dir.into_path().display().to_string()).unwrap();

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
        for blob in blobs {
            let output = node
                .blobs_add_bytes(blob, Arc::new(SetTagOption::auto()))
                .unwrap();
            hashes.push(output.hash);
        }

        let got_hashes = node.blobs_list().unwrap();
        assert_eq!(num_blobs, got_hashes.len());
        hashes_exist(&hashes, &got_hashes);

        let remove_hash = hashes.pop().unwrap();
        node.blobs_delete_blob(remove_hash.clone()).unwrap();

        let got_hashes = node.blobs_list().unwrap();
        assert_eq!(num_blobs - 1, got_hashes.len());
        hashes_exist(&hashes, &got_hashes);

        for hash in got_hashes {
            if remove_hash.equal(hash) {
                panic!("blob {} should have been removed", remove_hash);
            }
        }
    }

    async fn build_iroh_core(
        path: &std::path::Path,
        rt: &iroh::bytes::util::runtime::Handle,
    ) -> iroh::node::Node<iroh::bytes::store::flat::Store> {
        // TODO: store and load keypair
        let secret_key = iroh::net::key::SecretKey::generate();

        let docs_path = path.join("docs.db");
        let docs = iroh::sync::store::fs::Store::new(&docs_path).unwrap();

        // create a bao store for the iroh-bytes blobs
        let blob_path = path.join("blobs");
        tokio::fs::create_dir_all(&blob_path).await.unwrap();
        let db = iroh::bytes::store::flat::Store::load(&blob_path, &blob_path, &blob_path, &rt)
            .await
            .unwrap();

        iroh::node::Node::builder(db, docs)
            .bind_addr(iroh::node::DEFAULT_BIND_ADDR.into())
            .secret_key(secret_key)
            .runtime(rt)
            .spawn()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn iroh_core_blobs_add_get_bytes() {
        let tokio = tokio::runtime::Handle::current();
        let tpc = tokio_util::task::LocalPoolHandle::new(num_cpus::get());
        let rt = iroh::bytes::util::runtime::Handle::new(tokio, tpc);
        // temp dir
        let dir = tempfile::tempdir().unwrap();
        let node = build_iroh_core(dir.path(), &rt).await;
        let sizes = [1, 10, 100, 1000, 10000, 100000];
        let mut hashes = Vec::new();
        for size in sizes.iter() {
            let hash = iroh_core_blobs_add_get_bytes_size(&node, *size);
            hashes.push(hash)
        }
    }

    async fn iroh_core_blobs_add_get_bytes_size(
        node: &iroh::node::Node<iroh::bytes::store::flat::Store>,
        size: usize,
    ) -> iroh::bytes::Hash {
        let client = node.client();
        // create bytes
        let mut bytes = vec![0; size];
        rand::thread_rng().fill_bytes(&mut bytes);
        let bytes: bytes::Bytes = bytes.into();
        // add blob
        let tag = iroh::rpc_protocol::SetTagOption::Auto;
        let add_outcome = client.blobs.add_bytes(bytes.clone(), tag).await.unwrap();
        // check outcome
        assert_eq!(add_outcome.format, iroh::bytes::BlobFormat::Raw);
        assert_eq!(add_outcome.size, size as u64);
        // check size
        let hash = add_outcome.hash;
        let reader = client.blobs.read(hash).await.unwrap();
        let got_size = reader.size();
        assert_eq!(got_size, size as u64);
        //
        // get blob
        let got_bytes = client.blobs.read_to_bytes(hash).await.unwrap();
        assert_eq!(got_bytes.len(), size);
        assert_eq!(got_bytes, bytes);
        hash
    }

    #[tokio::test]
    async fn iroh_core_blobs_list_collections() {
        let tokio = tokio::runtime::Handle::current();
        let tpc = tokio_util::task::LocalPoolHandle::new(num_cpus::get());
        let rt = iroh::bytes::util::runtime::Handle::new(tokio, tpc);
        // iroh data dir
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = build_iroh_core(iroh_dir.path(), &rt).await;

        // collection dir
        let dir = tempfile::tempdir().unwrap();
        let num_blobs = 3;
        let blob_size = 100;
        for i in 0..num_blobs {
            let path = dir.path().join(i.to_string());
            let mut file = tokio::fs::File::create(path).await.unwrap();
            let mut bytes = vec![0; blob_size];
            rand::thread_rng().fill_bytes(&mut bytes);
            file.write_all(&bytes).await.unwrap()
        }

        let client = node.client();
        // ensure there are no blobs to start
        let blobs = client.blobs.list().await.unwrap().collect::<Vec<_>>().await;
        assert_eq!(0, blobs.len());

        let mut stream = client
            .blobs
            .add_from_path(
                dir.into_path(),
                false,
                iroh::rpc_protocol::SetTagOption::Auto,
                iroh::rpc_protocol::WrapOption::NoWrap,
            )
            .await
            .unwrap();

        let mut collection_hash = None;
        let mut collection_format = None;
        let mut hashes = Vec::new();

        while let Some(progress) = stream.next().await {
            let progress = progress.unwrap();
            match progress {
                iroh::bytes::provider::AddProgress::AllDone { hash, format, .. } => {
                    collection_hash = Some(hash);
                    collection_format = Some(format);
                }
                iroh::bytes::provider::AddProgress::Abort(err) => {
                    panic!("{}", err);
                }
                iroh::bytes::provider::AddProgress::Done { hash, .. } => hashes.push(hash),
                _ => {}
            }
        }

        let collection_hash = collection_hash.unwrap();
        let collection_format = collection_format.unwrap();

        assert_eq!(iroh::rpc_protocol::BlobFormat::HashSeq, collection_format);

        let collections = client
            .blobs
            .list_collections()
            .await
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        assert_eq!(1, collections.len());
        assert_eq!(collections[0].hash, collection_hash);
        // add length for the metadata blob
        let total_blobs_count = hashes.len() + 1;
        assert_eq!(
            collections[0].total_blobs_count.unwrap(),
            total_blobs_count as u64
        );
        // this is always `None`
        // assert_eq!(collections[0].total_blobs_size.unwrap(), 300 as u64);
    }
}
