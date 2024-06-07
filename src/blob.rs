use std::{path::PathBuf, str::FromStr, sync::Arc, sync::RwLock, time::Duration};

use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::node::IrohNode;
use crate::ticket::AddrInfoOptions;
use crate::{block_on, IrohError, NodeAddr};

impl IrohNode {
    /// List all complete blobs.
    ///
    /// Note: this allocates for each `BlobListResponse`, if you have many `BlobListReponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn blobs_list(&self) -> Result<Vec<Arc<Hash>>, IrohError> {
        block_on(&self.rt(), async {
            let response = self
                .sync_client
                .blobs()
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
    /// Method only exists in FFI
    pub fn blobs_size(&self, hash: &Hash) -> Result<u64, IrohError> {
        block_on(&self.rt(), async {
            let r = self
                .sync_client
                .blobs()
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
        block_on(&self.rt(), async {
            self.sync_client
                .blobs()
                .read_to_bytes(hash.0)
                .await
                .map(|b| b.to_vec())
                .map_err(IrohError::blobs)
        })
    }

    /// Read all bytes of single blob at `offset` for length `len`.
    ///
    /// This allocates a buffer for the full length `len`. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_at_to_bytes`].
    pub fn blobs_read_at_to_bytes(
        &self,
        hash: Arc<Hash>,
        offset: u64,
        len: Option<u64>,
    ) -> Result<Vec<u8>, IrohError> {
        let len = match len {
            None => None,
            Some(l) => Some(usize::try_from(l).map_err(IrohError::blobs)?),
        };
        block_on(&self.rt(), async {
            self.sync_client
                .blobs()
                .read_at_to_bytes(hash.0, offset, len)
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
        block_on(&self.rt(), async {
            let mut stream = self
                .sync_client
                .blobs()
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
                cb.progress(Arc::new(progress.into()))?;
            }
            Ok(())
        })
    }

    /// Export the blob contents to a file path
    /// The `path` field is expected to be the absolute path.
    pub fn blobs_write_to_path(&self, hash: Arc<Hash>, path: String) -> Result<(), IrohError> {
        block_on(&self.rt(), async {
            let mut reader = self
                .sync_client
                .blobs()
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
    pub fn blobs_add_bytes(&self, bytes: Vec<u8>) -> Result<BlobAddOutcome, IrohError> {
        block_on(&self.rt(), async {
            self.sync_client
                .blobs()
                .add_bytes(bytes)
                .await
                .map(|outcome| outcome.into())
                .map_err(IrohError::blobs)
        })
    }

    /// Download a blob from another node and add it to the local database.
    pub fn blobs_download(
        &self,
        hash: Arc<Hash>,
        opts: Arc<BlobDownloadOptions>,
        cb: Box<dyn DownloadCallback>,
    ) -> Result<(), IrohError> {
        block_on(&self.rt(), async {
            let mut stream = self
                .sync_client
                .blobs()
                .download_with_opts(hash.0, opts.0.clone())
                .await
                .map_err(IrohError::blobs)?;
            while let Some(progress) = stream.next().await {
                let progress = progress.map_err(IrohError::blobs)?;
                cb.progress(Arc::new(progress.into()))?;
            }
            Ok(())
        })
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
    pub fn blobs_export(
        &self,
        hash: Arc<Hash>,
        destination: String,
        format: BlobExportFormat,
        mode: BlobExportMode,
    ) -> Result<(), IrohError> {
        block_on(&self.rt(), async {
            let destination: PathBuf = destination.into();
            if let Some(dir) = destination.parent() {
                tokio::fs::create_dir_all(dir)
                    .await
                    .map_err(IrohError::blobs)?;
            }

            let stream = self
                .sync_client
                .blobs()
                .export(hash.0, destination, format.into(), mode.into())
                .await
                .map_err(IrohError::blobs)?;

            stream.finish().await.map_err(IrohError::blobs)?;

            Ok(())
        })
    }

    /// Create a ticket for sharing a blob from this node.
    pub fn blobs_share(
        &self,
        hash: Arc<Hash>,
        blob_format: BlobFormat,
        ticket_options: AddrInfoOptions,
    ) -> Result<String, IrohError> {
        block_on(&self.rt(), async {
            let ticket = self
                .sync_client
                .blobs()
                .share(hash.0, blob_format.into(), ticket_options.into())
                .await
                .map_err(IrohError::blobs)?;
            Ok(ticket.to_string())
        })
    }

    /// List all incomplete (partial) blobs.
    ///
    /// Note: this allocates for each `BlobListIncompleteResponse`, if you have many `BlobListIncompleteResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn blobs_list_incomplete(&self) -> Result<Vec<IncompleteBlobInfo>, IrohError> {
        block_on(&self.rt(), async {
            let blobs = self
                .sync_client
                .blobs()
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
    pub fn blobs_list_collections(&self) -> Result<Vec<CollectionInfo>, IrohError> {
        block_on(&self.rt(), async {
            let blobs = self
                .sync_client
                .blobs()
                .list_collections()
                .map_err(IrohError::blobs)?
                .map_ok(|res| res.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::blobs)?;
            Ok(blobs)
        })
    }

    /// Read the content of a collection
    pub fn blobs_get_collection(&self, hash: Arc<Hash>) -> Result<Arc<Collection>, IrohError> {
        block_on(&self.rt(), async {
            let collection = self
                .sync_client
                .blobs()
                .get_collection(hash.0)
                .await
                .map_err(IrohError::collection)?;
            Ok(Arc::new(collection.into()))
        })
    }

    /// Create a collection from already existing blobs.
    ///
    /// To automatically clear the tags for the passed in blobs you can set
    /// `tags_to_delete` on those tags, and they will be deleted once the collection is created.
    pub fn blobs_create_collection(
        &self,
        collection: Arc<Collection>,
        tag: Arc<SetTagOption>,
        tags_to_delete: Vec<String>,
    ) -> Result<HashAndTag, IrohError> {
        block_on(&self.rt(), async {
            let collection = collection.0.read().map_err(IrohError::collection)?.clone();
            let (hash, tag) = self
                .sync_client
                .blobs()
                .create_collection(
                    collection,
                    (*tag).clone().into(),
                    tags_to_delete
                        .into_iter()
                        .map(iroh::blobs::Tag::from)
                        .collect(),
                )
                .await
                .map_err(IrohError::blobs)?;
            Ok(HashAndTag {
                hash: Arc::new(hash.into()),
                tag: tag.0.to_vec(),
            })
        })
    }

    /// Delete a blob.
    pub fn blobs_delete_blob(&self, hash: Arc<Hash>) -> Result<(), IrohError> {
        block_on(&self.rt(), async {
            let mut tags = self
                .sync_client
                .tags()
                .list()
                .await
                .map_err(IrohError::blobs)?;
            let mut name = None;
            while let Some(tag) = tags.next().await {
                let tag = tag.map_err(IrohError::blobs)?;
                if tag.hash == hash.0 {
                    name = Some(tag.name);
                }
            }

            if let Some(name) = name {
                self.sync_client
                    .tags()
                    .delete(name)
                    .await
                    .map_err(IrohError::blobs)?;
                self.sync_client
                    .blobs()
                    .delete_blob((*hash).clone().0)
                    .await
                    .map_err(IrohError::blobs)?;
            }

            Ok(())
        })
    }
}

/// The Hash and associated tag of a newly created collection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashAndTag {
    /// The hash of the collection
    pub hash: Arc<Hash>,
    /// The tag of the collection
    pub tag: Vec<u8>,
}

/// Outcome of a blob add operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl From<iroh::client::blobs::AddOutcome> for BlobAddOutcome {
    fn from(value: iroh::client::blobs::AddOutcome) -> Self {
        BlobAddOutcome {
            hash: Arc::new(value.hash.into()),
            format: value.format.into(),
            size: value.size,
            tag: value.tag.0.to_vec(),
        }
    }
}

/// An option for commands that allow setting a Tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetTagOption {
    /// A tag will be automatically generated
    Auto,
    /// The tag is explicitly vecnamed
    Named(Vec<u8>),
}

impl SetTagOption {
    /// Indicate you want an automatically generated tag
    pub fn auto() -> Self {
        SetTagOption::Auto
    }

    /// Indicate you want a named tag
    pub fn named(tag: Vec<u8>) -> Self {
        SetTagOption::Named(tag)
    }
}

impl From<SetTagOption> for iroh::blobs::util::SetTagOption {
    fn from(value: SetTagOption) -> Self {
        match value {
            SetTagOption::Auto => iroh::blobs::util::SetTagOption::Auto,
            SetTagOption::Named(tag) => {
                iroh::blobs::util::SetTagOption::Named(iroh::blobs::Tag(bytes::Bytes::from(tag)))
            }
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

impl From<WrapOption> for iroh::client::blobs::WrapOption {
    fn from(value: WrapOption) -> Self {
        match value {
            WrapOption::NoWrap => iroh::client::blobs::WrapOption::NoWrap,
            WrapOption::Wrap { name } => iroh::client::blobs::WrapOption::Wrap { name },
        }
    }
}

/// Hash type used throughout Iroh. A blake3 hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub(crate) iroh::blobs::Hash);

impl From<iroh::blobs::Hash> for Hash {
    fn from(h: iroh::blobs::Hash) -> Self {
        Hash(h)
    }
}

impl Hash {
    /// Calculate the hash of the provide bytes.
    pub fn new(buf: Vec<u8>) -> Self {
        Hash(iroh::blobs::Hash::new(buf))
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
        Ok(Hash(iroh::blobs::Hash::from_bytes(bytes)))
    }

    /// Make a Hash from hex string
    pub fn from_string(s: String) -> Result<Self, IrohError> {
        match iroh::blobs::Hash::from_str(&s) {
            Ok(key) => Ok(key.into()),
            Err(err) => Err(IrohError::hash(err)),
        }
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

impl From<Hash> for iroh::blobs::Hash {
    fn from(value: Hash) -> Self {
        value.0
    }
}

/// The `progress` method will be called for each `AddProgress` event that is
/// emitted during a `node.blobs_add_from_path`. Use the `AddProgress.type()`
/// method to check the `AddProgressType`
pub trait AddCallback: Send + Sync + 'static {
    fn progress(&self, progress: Arc<AddProgress>) -> Result<(), IrohError>;
}

/// The different types of AddProgress events
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddProgressFound {
    /// A new unique id for this entry.
    pub id: u64,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: u64,
}

/// An AddProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// An AddProgress event indicated we are done with `id` and now have a hash `hash`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddProgressDone {
    /// The unique id of the entry.
    pub id: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// An AddProgress event indicating we are done with the the whole operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddProgressAllDone {
    /// The hash of the created data.
    pub hash: Arc<Hash>,
    /// The format of the added data.
    pub format: BlobFormat,
    /// The tag of the added data.
    pub tag: Vec<u8>,
}

/// An AddProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddProgressAbort {
    pub error: String,
}

/// Progress updates for the add operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl From<iroh::blobs::provider::AddProgress> for AddProgress {
    fn from(value: iroh::blobs::provider::AddProgress) -> Self {
        match value {
            iroh::blobs::provider::AddProgress::Found { id, name, size } => {
                AddProgress::Found(AddProgressFound { id, name, size })
            }
            iroh::blobs::provider::AddProgress::Progress { id, offset } => {
                AddProgress::Progress(AddProgressProgress { id, offset })
            }
            iroh::blobs::provider::AddProgress::Done { id, hash } => {
                AddProgress::Done(AddProgressDone {
                    id,
                    hash: Arc::new(hash.into()),
                })
            }
            iroh::blobs::provider::AddProgress::AllDone { hash, format, tag } => {
                AddProgress::AllDone(AddProgressAllDone {
                    hash: Arc::new(hash.into()),
                    format: format.into(),
                    tag: tag.0.to_vec(),
                })
            }
            iroh::blobs::provider::AddProgress::Abort(err) => {
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobFormat {
    /// Raw blob
    Raw,
    /// A sequence of BLAKE3 hashes
    HashSeq,
}

impl From<iroh::blobs::BlobFormat> for BlobFormat {
    fn from(value: iroh::blobs::BlobFormat) -> Self {
        match value {
            iroh::blobs::BlobFormat::Raw => BlobFormat::Raw,
            iroh::blobs::BlobFormat::HashSeq => BlobFormat::HashSeq,
        }
    }
}

impl From<BlobFormat> for iroh::blobs::BlobFormat {
    fn from(value: BlobFormat) -> Self {
        match value {
            BlobFormat::Raw => iroh::blobs::BlobFormat::Raw,
            BlobFormat::HashSeq => iroh::blobs::BlobFormat::HashSeq,
        }
    }
}

/// Options to download  data specified by the hash.
pub struct BlobDownloadOptions(iroh::client::blobs::DownloadOptions);
impl BlobDownloadOptions {
    /// Create a BlobDownloadRequest
    pub fn new(
        format: BlobFormat,
        node: Arc<NodeAddr>,
        tag: Arc<SetTagOption>,
    ) -> Result<Self, IrohError> {
        Ok(BlobDownloadOptions(iroh::client::blobs::DownloadOptions {
            format: format.into(),
            nodes: vec![(*node).clone().try_into()?],
            tag: (*tag).clone().into(),
            mode: iroh::client::blobs::DownloadMode::Direct,
        }))
    }
}

impl From<iroh::client::blobs::DownloadOptions> for BlobDownloadOptions {
    fn from(value: iroh::client::blobs::DownloadOptions) -> Self {
        BlobDownloadOptions(value)
    }
}

/// The expected format of a hash being exported.
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

impl From<BlobExportFormat> for iroh::blobs::store::ExportFormat {
    fn from(value: BlobExportFormat) -> Self {
        match value {
            BlobExportFormat::Blob => iroh::blobs::store::ExportFormat::Blob,
            BlobExportFormat::Collection => iroh::blobs::store::ExportFormat::Collection,
        }
    }
}

/// The export mode describes how files will be exported.
///
/// This is a hint to the import trait method. For some implementations, this
/// does not make any sense. E.g. an in memory implementation will always have
/// to copy the file into memory. Also, a disk based implementation might choose
/// to copy small files even if the mode is `Reference`.
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

impl From<BlobExportMode> for iroh::blobs::store::ExportMode {
    fn from(value: BlobExportMode) -> Self {
        match value {
            BlobExportMode::Copy => iroh::blobs::store::ExportMode::Copy,
            BlobExportMode::TryReference => iroh::blobs::store::ExportMode::TryReference,
        }
    }
}

/// The `progress` method will be called for each `DownloadProgress` event that is emitted during
/// a `node.blobs_download`. Use the `DownloadProgress.type()` method to check the()
/// `DownloadProgressType` of the event.
pub trait DownloadCallback: Send + Sync + 'static {
    fn progress(&self, progress: Arc<DownloadProgress>) -> Result<(), IrohError>;
}

/// The different types of DownloadProgress events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgressFoundHashSeq {
    /// Number of children in the collection, if known.
    pub children: u64,
    /// The hash of the entry.
    pub hash: Arc<Hash>,
}

/// A DownloadProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgressProgress {
    /// The unique id of the entry.
    pub id: u64,
    /// The offset of the progress, in bytes.
    pub offset: u64,
}

/// A DownloadProgress event indicated we are done with `id`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgressDone {
    /// The unique id of the entry.
    pub id: u64,
}

/// A DownloadProgress event indicating we are done with the whole operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgressAllDone {
    /// The number of bytes written
    pub bytes_written: u64,
    /// The number of bytes read
    pub bytes_read: u64,
    /// The time it took to transfer the data
    pub elapsed: Duration,
}

/// A DownloadProgress event indicating we got an error and need to abort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgressAbort {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl From<iroh::blobs::get::db::DownloadProgress> for DownloadProgress {
    fn from(value: iroh::blobs::get::db::DownloadProgress) -> Self {
        match value {
            iroh::blobs::get::db::DownloadProgress::InitialState(transfer_state) => {
                DownloadProgress::InitialState(DownloadProgressInitialState {
                    connected: transfer_state.connected,
                })
            }
            iroh::blobs::get::db::DownloadProgress::FoundLocal {
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
            iroh::blobs::get::db::DownloadProgress::Connected => DownloadProgress::Connected,
            iroh::blobs::get::db::DownloadProgress::Found {
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
            iroh::blobs::get::db::DownloadProgress::FoundHashSeq { hash, children } => {
                DownloadProgress::FoundHashSeq(DownloadProgressFoundHashSeq {
                    hash: Arc::new(hash.into()),
                    children,
                })
            }
            iroh::blobs::get::db::DownloadProgress::Progress { id, offset } => {
                DownloadProgress::Progress(DownloadProgressProgress { id, offset })
            }
            iroh::blobs::get::db::DownloadProgress::Done { id } => {
                DownloadProgress::Done(DownloadProgressDone { id })
            }
            iroh::blobs::get::db::DownloadProgress::AllDone(stats) => {
                DownloadProgress::AllDone(DownloadProgressAllDone {
                    bytes_written: stats.bytes_written,
                    bytes_read: stats.bytes_read,
                    elapsed: stats.elapsed,
                })
            }
            iroh::blobs::get::db::DownloadProgress::Abort(err) => {
                DownloadProgress::Abort(DownloadProgressAbort {
                    error: err.to_string(),
                })
            }
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeSpec(pub(crate) iroh::blobs::protocol::RangeSpec);

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

impl From<iroh::blobs::protocol::RangeSpec> for RangeSpec {
    fn from(h: iroh::blobs::protocol::RangeSpec) -> Self {
        RangeSpec(h)
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone)]
pub struct BlobInfo {
    /// Location of the blob
    pub path: String,
    /// The hash of the blob
    pub hash: Arc<Hash>,
    /// The size of the blob
    pub size: u64,
}

impl From<iroh::client::blobs::BlobInfo> for BlobInfo {
    fn from(value: iroh::client::blobs::BlobInfo) -> Self {
        BlobInfo {
            path: value.path,
            hash: Arc::new(value.hash.into()),
            size: value.size,
        }
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteBlobInfo {
    /// The size we got
    pub size: u64,
    /// The size we expect
    pub expected_size: u64,
    /// The hash of the blob
    pub hash: Arc<Hash>,
}

impl From<iroh::client::blobs::IncompleteBlobInfo> for IncompleteBlobInfo {
    fn from(value: iroh::client::blobs::IncompleteBlobInfo) -> Self {
        IncompleteBlobInfo {
            size: value.size,
            expected_size: value.expected_size,
            hash: Arc::new(value.hash.into()),
        }
    }
}

/// A response to a list collections request
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl From<iroh::client::blobs::CollectionInfo> for CollectionInfo {
    fn from(value: iroh::client::blobs::CollectionInfo) -> Self {
        CollectionInfo {
            tag: value.tag.0.to_vec(),
            hash: Arc::new(value.hash.into()),
            total_blobs_count: value.total_blobs_count,
            total_blobs_size: value.total_blobs_size,
        }
    }
}

/// A collection of blobs
#[derive(Debug)]
pub struct Collection(pub(crate) RwLock<iroh::blobs::format::collection::Collection>);

impl From<iroh::blobs::format::collection::Collection> for Collection {
    fn from(value: iroh::blobs::format::collection::Collection) -> Self {
        Collection(RwLock::new(value))
    }
}

impl From<Collection> for iroh::blobs::format::collection::Collection {
    fn from(value: Collection) -> Self {
        let col = value.0.read().expect("Collection lock poisoned");
        col.clone()
    }
}

impl Collection {
    /// Create a new empty collection
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Collection(RwLock::new(
            iroh::blobs::format::collection::Collection::default(),
        ))
    }

    /// Add the given blob to the collection
    pub fn push(&self, name: String, hash: &Hash) -> Result<(), IrohError> {
        self.0
            .write()
            .map_err(IrohError::collection)?
            .push(name, hash.0);
        Ok(())
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> Result<bool, IrohError> {
        Ok(self.0.read().map_err(IrohError::collection)?.is_empty())
    }

    /// Get the names of the blobs in this collection
    pub fn names(&self) -> Result<Vec<String>, IrohError> {
        Ok(self
            .0
            .read()
            .map_err(IrohError::collection)?
            .iter()
            .map(|(name, _)| name.clone())
            .collect())
    }

    /// Get the links to the blobs in this collection
    pub fn links(&self) -> Result<Vec<Arc<Hash>>, IrohError> {
        Ok(self
            .0
            .read()
            .map_err(IrohError::collection)?
            .iter()
            .map(|(_, hash)| Arc::new(Hash(*hash)))
            .collect())
    }

    /// Get the blobs associated with this collection
    pub fn blobs(&self) -> Result<Vec<LinkAndName>, IrohError> {
        Ok(self
            .0
            .read()
            .map_err(IrohError::collection)?
            .iter()
            .map(|(name, hash)| LinkAndName {
                name: name.clone(),
                link: Arc::new(Hash(*hash)),
            })
            .collect())
    }

    /// Returns the number of blobs in this collection
    pub fn len(&self) -> Result<u64, IrohError> {
        Ok(self.0.read().map_err(IrohError::collection)?.len() as _)
    }
}

/// `LinkAndName` includes a name and a hash for a blob in a collection
#[derive(Clone, Debug)]
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
    use crate::node::IrohNode;
    use crate::NodeOptions;
    use bytes::Bytes;
    use rand::RngCore;
    use tokio::io::AsyncWriteExt;
    use tracing_subscriber::FmtSubscriber;

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

    #[test]
    fn test_blobs_add_get_bytes() {
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
        let add_outcome = node.blobs_add_bytes(bytes.to_vec()).unwrap();
        // check outcome
        assert_eq!(add_outcome.format, BlobFormat::Raw);
        assert_eq!(add_outcome.size, size as u64);
        // check size
        let hash = add_outcome.hash;
        let got_size = node.blobs_size(&hash).unwrap();
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
            let hash = output.hash.as_ref().cloned().unwrap();
            let format = output.format.as_ref().cloned().unwrap();
            (hash, format)
        };

        // check outcome info is as expected
        assert_eq!(BlobFormat::Raw, format);

        // check we get the expected size from the hash
        let got_size = node.blobs_size(&hash).unwrap();
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
        let node = IrohNode::new(iroh_dir.into_path().display().to_string()).unwrap();

        // ensure there are no blobs to start
        let blobs = node.blobs_list().unwrap();
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

        let blobs = node.blobs_list().unwrap();
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

    #[test]
    fn test_list_and_delete() {
        setup_logging();

        let iroh_dir = tempfile::tempdir().unwrap();
        // we're going to use a very fast GC interval to get this test to delete stuff aggressively
        let opts = NodeOptions {
            gc_interval_millis: Some(100),
        };
        let node =
            IrohNode::with_options(iroh_dir.into_path().display().to_string(), opts).unwrap();

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
            let output = node.blobs_add_bytes(blob).unwrap();
            hashes.push(output.hash);
            tags.push(output.tag);
        }

        let got_hashes = node.blobs_list().unwrap();
        assert_eq!(num_blobs, got_hashes.len());
        hashes_exist(&hashes, &got_hashes);

        let remove_hash = hashes.pop().unwrap();
        let remove_tag = tags.pop().unwrap();
        // delete the tag for the first blob
        node.tags_delete(remove_tag).unwrap();
        // wait for GC to clear the blob. windows test runner is slow & needs like 500ms
        std::thread::sleep(Duration::from_millis(500));

        let got_hashes = node.blobs_list().unwrap();
        assert_eq!(num_blobs - 1, got_hashes.len());
        hashes_exist(&hashes, &got_hashes);

        for hash in got_hashes {
            if remove_hash.equal(&hash) {
                panic!("blob {} should have been removed", remove_hash);
            }
        }
    }

    async fn build_iroh_core(
        path: &std::path::Path,
    ) -> iroh::node::Node<iroh::blobs::store::fs::Store> {
        iroh::node::Node::persistent(path)
            .await
            .unwrap()
            .spawn()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn iroh_core_blobs_add_get_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let node = build_iroh_core(dir.path()).await;
        let sizes = [1, 10, 100, 1000, 10000, 100000];
        let mut hashes = Vec::new();
        for size in sizes.iter() {
            let hash = iroh_core_blobs_add_get_bytes_size(&node, *size);
            hashes.push(hash)
        }
    }

    async fn iroh_core_blobs_add_get_bytes_size(
        node: &iroh::node::Node<iroh::blobs::store::fs::Store>,
        size: usize,
    ) -> iroh::blobs::Hash {
        let client = node.client();
        // create bytes
        let mut bytes = vec![0; size];
        rand::thread_rng().fill_bytes(&mut bytes);
        let bytes: Bytes = bytes.into();
        // add blob
        let add_outcome = client.blobs().add_bytes(bytes.clone()).await.unwrap();
        // check outcome
        assert_eq!(add_outcome.format, iroh::blobs::BlobFormat::Raw);
        assert_eq!(add_outcome.size, size as u64);
        // check size
        let hash = add_outcome.hash;
        let reader = client.blobs().read(hash).await.unwrap();
        let got_size = reader.size();
        assert_eq!(got_size, size as u64);
        //
        // get blob
        let got_bytes = client.blobs().read_to_bytes(hash).await.unwrap();
        assert_eq!(got_bytes.len(), size);
        assert_eq!(got_bytes, bytes);
        hash
    }

    #[tokio::test]
    async fn iroh_core_blobs_list_collections() {
        let iroh_dir = tempfile::tempdir().unwrap();
        let node = build_iroh_core(iroh_dir.path()).await;

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
        let blobs = client
            .blobs()
            .list()
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        assert_eq!(0, blobs.len());

        let mut stream = client
            .blobs()
            .add_from_path(
                dir.into_path(),
                false,
                iroh::blobs::util::SetTagOption::Auto,
                iroh::client::blobs::WrapOption::NoWrap,
            )
            .await
            .unwrap();

        let mut collection_hash = None;
        let mut collection_format = None;
        let mut hashes = Vec::new();

        while let Some(progress) = stream.next().await {
            let progress = progress.unwrap();
            match progress {
                iroh::blobs::provider::AddProgress::AllDone { hash, format, .. } => {
                    collection_hash = Some(hash);
                    collection_format = Some(format);
                }
                iroh::blobs::provider::AddProgress::Abort(err) => {
                    panic!("{}", err);
                }
                iroh::blobs::provider::AddProgress::Done { hash, .. } => hashes.push(hash),
                _ => {}
            }
        }

        let collection_hash = collection_hash.unwrap();
        let collection_format = collection_format.unwrap();

        assert_eq!(iroh::blobs::BlobFormat::HashSeq, collection_format);

        let collections = client
            .blobs()
            .list_collections()
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

    pub fn setup_logging() {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(format!(
                "{}=debug",
                env!("CARGO_PKG_NAME").replace('-', "_")
            ))
            .compact()
            .finish();

        tracing::subscriber::set_global_default(subscriber).ok();
    }
}
