use std::{path::PathBuf, str::FromStr, sync::RwLock};

use futures::{StreamExt, TryStreamExt};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;

use crate::{node::Iroh, AddrInfoOptions, BlobTicket, NodeAddr};

/// Iroh blobs client.
#[napi]
pub struct Blobs {
    node: Iroh,
}

#[napi]
impl Iroh {
    /// Access to blob specific funtionaliy.
    #[napi(getter)]
    pub fn blobs(&self) -> Blobs {
        Blobs { node: self.clone() }
    }
}

impl Blobs {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.inner_client()
    }
}

#[napi]
impl Blobs {
    /// List all complete blobs.
    ///
    /// Note: this allocates for each `BlobListResponse`, if you have many `BlobListReponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi]
    pub async fn list(&self) -> Result<Vec<Hash>> {
        let response = self.client().blobs().list().await?;

        let hashes: Vec<Hash> = response.map_ok(|i| i.hash.into()).try_collect().await?;

        Ok(hashes)
    }

    /// Get the size information on a single blob.
    ///
    /// Method only exists in FFI
    #[napi]
    pub async fn size(&self, hash: String) -> Result<u64> {
        let r = self
            .client()
            .blobs()
            .read(hash.parse().map_err(anyhow::Error::from)?)
            .await?;
        Ok(r.size())
    }

    /// Read all bytes of single blob.
    ///
    /// This allocates a buffer for the full blob. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_to_bytes`].
    #[napi]
    pub async fn read_to_bytes(&self, hash: String) -> Result<Vec<u8>> {
        let res = self
            .client()
            .blobs()
            .read_to_bytes(hash.parse().map_err(anyhow::Error::from)?)
            .await
            .map(|b| b.to_vec())?;
        Ok(res)
    }

    /// Read all bytes of single blob at `offset` for length `len`.
    ///
    /// This allocates a buffer for the full length `len`. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_at_to_bytes`].
    #[napi]
    pub async fn read_at_to_bytes(
        &self,
        hash: String,
        offset: BigInt,
        len: Option<BigInt>,
    ) -> Result<Vec<u8>> {
        let len = match len {
            None => None,
            Some(l) => Some(usize::try_from(l.get_u64().1).map_err(anyhow::Error::from)?),
        };
        let res = self
            .client()
            .blobs()
            .read_at_to_bytes(
                hash.parse().map_err(anyhow::Error::from)?,
                offset.get_u64().1,
                len,
            )
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
    #[napi]
    pub async fn add_from_path(
        &self,
        path: String,
        in_place: bool,
        tag: &SetTagOption,
        wrap: WrapOption,
        cb: ThreadsafeFunction<AddProgress, ()>,
    ) -> Result<()> {
        let mut stream = self
            .client()
            .blobs()
            .add_from_path(path.into(), in_place, tag.into(), wrap.into())
            .await?;
        while let Some(progress) = stream.next().await {
            let progress = AddProgress::convert(progress);
            cb.call_async(progress).await?;
        }
        Ok(())
    }

    /// Export the blob contents to a file path
    /// The `path` field is expected to be the absolute path.
    #[napi]
    pub async fn write_to_path(&self, hash: String, path: String) -> Result<()> {
        let mut reader = self
            .client()
            .blobs()
            .read(hash.parse().map_err(anyhow::Error::from)?)
            .await?;
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
    #[napi]
    pub async fn add_bytes(&self, bytes: Vec<u8>) -> Result<BlobAddOutcome> {
        let res = self.client().blobs().add_bytes(bytes).await?;
        Ok(res.into())
    }

    /// Write a blob by passing bytes, setting an explicit tag name.
    #[napi]
    pub async fn add_bytes_named(&self, bytes: Vec<u8>, name: String) -> Result<BlobAddOutcome> {
        let res = self
            .client()
            .blobs()
            .add_bytes_named(bytes, iroh::blobs::Tag(name.into()))
            .await?;
        Ok(res.into())
    }

    /// Download a blob from another node and add it to the local database.
    #[napi]
    pub async fn download(
        &self,
        hash: String,
        opts: &BlobDownloadOptions,
        cb: ThreadsafeFunction<DownloadProgress, ()>,
    ) -> Result<()> {
        let mut stream = self
            .client()
            .blobs()
            .download_with_opts(hash.parse().map_err(anyhow::Error::from)?, opts.0.clone())
            .await?;
        while let Some(progress) = stream.next().await {
            let progress = DownloadProgress::convert(progress);
            // The callback failing is not fatal
            if let Err(err) = cb.call_async(progress).await {
                tracing::warn!("download callback failed: {:?}", err);
            }
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
    #[napi]
    pub async fn export(
        &self,
        hash: String,
        destination: String,
        format: BlobExportFormat,
        mode: BlobExportMode,
    ) -> Result<()> {
        let destination: PathBuf = destination.into();
        if let Some(dir) = destination.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(anyhow::Error::from)?;
        }

        let stream = self
            .client()
            .blobs()
            .export(
                hash.parse().map_err(anyhow::Error::from)?,
                destination,
                format.into(),
                mode.into(),
            )
            .await?;

        stream.finish().await?;

        Ok(())
    }

    /// Create a ticket for sharing a blob from this node.
    #[napi]
    pub async fn share(
        &self,
        hash: String,
        blob_format: BlobFormat,
        ticket_options: AddrInfoOptions,
    ) -> Result<BlobTicket> {
        let ticket = self
            .client()
            .blobs()
            .share(
                hash.parse().map_err(anyhow::Error::from)?,
                blob_format.into(),
                ticket_options.into(),
            )
            .await?;
        Ok(ticket.into())
    }

    /// List all incomplete (partial) blobs.
    ///
    /// Note: this allocates for each `BlobListIncompleteResponse`, if you have many `BlobListIncompleteResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi]
    pub async fn list_incomplete(&self) -> Result<Vec<IncompleteBlobInfo>> {
        let blobs = self
            .client()
            .blobs()
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
    #[napi]
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let blobs = self
            .client()
            .blobs()
            .list_collections()?
            .map_ok(|res| res.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(blobs)
    }

    /// Read the content of a collection
    #[napi]
    pub async fn get_collection(&self, hash: String) -> Result<Collection> {
        let collection = self
            .client()
            .blobs()
            .get_collection(hash.parse().map_err(anyhow::Error::from)?)
            .await?;

        Ok(collection.into())
    }

    /// Create a collection from already existing blobs.
    ///
    /// To automatically clear the tags for the passed in blobs you can set
    /// `tags_to_delete` on those tags, and they will be deleted once the collection is created.
    #[napi]
    pub async fn create_collection(
        &self,
        collection: &Collection,
        tag: &SetTagOption,
        tags_to_delete: Vec<String>,
    ) -> Result<HashAndTag> {
        let collection = collection.0.read().unwrap().clone();
        let (hash, tag) = self
            .client()
            .blobs()
            .create_collection(
                collection,
                tag.into(),
                tags_to_delete
                    .into_iter()
                    .map(iroh::blobs::Tag::from)
                    .collect(),
            )
            .await?;

        Ok(HashAndTag {
            hash: hash.to_string(),
            tag: tag.0.to_vec(),
        })
    }

    /// Delete a blob.
    #[napi]
    pub async fn delete_blob(&self, hash: String) -> Result<()> {
        let mut tags = self.client().tags().list().await?;
        let hash: iroh::blobs::Hash = hash.parse().map_err(anyhow::Error::from)?;

        let mut name = None;
        while let Some(tag) = tags.next().await {
            let tag = tag?;
            if tag.hash == hash {
                name = Some(tag.name);
            }
        }

        if let Some(name) = name {
            self.client().tags().delete(name).await?;
            self.client().blobs().delete_blob(hash).await?;
        }

        Ok(())
    }
}

/// The Hash and associated tag of a newly created collection
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi(object)]
pub struct HashAndTag {
    /// The hash of the collection
    pub hash: String,
    /// The tag of the collection
    pub tag: Vec<u8>,
}

/// Outcome of a blob add operation.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct BlobAddOutcome {
    /// The hash of the blob
    pub hash: String,
    /// The format the blob
    pub format: BlobFormat,
    /// The size of the blob
    pub size: BigInt,
    /// The tag of the blob
    pub tag: Vec<u8>,
}

impl From<iroh::client::blobs::AddOutcome> for BlobAddOutcome {
    fn from(value: iroh::client::blobs::AddOutcome) -> Self {
        BlobAddOutcome {
            hash: value.hash.to_string(),
            format: value.format.into(),
            size: value.size.into(),
            tag: value.tag.0.to_vec(),
        }
    }
}

/// An option for commands that allow setting a Tag
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct SetTagOption {
    /// A tag will be automatically generated
    #[napi(readonly)]
    pub auto: bool,
    /// The tag is explicitly vecnamed
    #[napi(readonly)]
    pub name: Option<Vec<u8>>,
}

#[napi]
impl SetTagOption {
    /// Indicate you want an automatically generated tag
    #[napi(factory)]
    pub fn auto() -> Self {
        SetTagOption {
            auto: true,
            name: None,
        }
    }

    /// Indicate you want a named tag
    #[napi(factory)]
    pub fn named(tag: Vec<u8>) -> Self {
        SetTagOption {
            auto: false,
            name: Some(tag),
        }
    }
}

impl From<&SetTagOption> for iroh::blobs::util::SetTagOption {
    fn from(value: &SetTagOption) -> Self {
        if let Some(ref tag) = value.name {
            iroh::blobs::util::SetTagOption::Named(iroh::blobs::Tag(bytes::Bytes::from(
                tag.clone(),
            )))
        } else if value.auto {
            iroh::blobs::util::SetTagOption::Auto
        } else {
            panic!("invalid settagoption state");
        }
    }
}

/// Whether to wrap the added data in a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi(object)]
pub struct WrapOption {
    /// Wrap the file or directory in a colletion.
    pub wrap: bool,
    /// Override the filename in the wrapping collection.
    pub wrap_override: Option<String>,
}

impl From<WrapOption> for iroh::client::blobs::WrapOption {
    fn from(value: WrapOption) -> Self {
        if value.wrap {
            iroh::client::blobs::WrapOption::Wrap {
                name: value.wrap_override,
            }
        } else {
            iroh::client::blobs::WrapOption::NoWrap
        }
    }
}

/// Hash type used throughout Iroh. A blake3 hash.
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct Hash {
    /// The base32 representation of the hash.
    #[napi(readonly)]
    pub value: String,
}

impl From<iroh::blobs::Hash> for Hash {
    fn from(h: iroh::blobs::Hash) -> Self {
        Hash {
            value: h.to_string(),
        }
    }
}

#[napi]
impl Hash {
    /// Calculate the hash of the provide bytes.
    #[napi(constructor)]
    pub fn new(buf: Vec<u8>) -> Self {
        Hash {
            value: iroh::blobs::Hash::new(buf).to_string(),
        }
    }

    /// Checks if the other hash is equal to this instance.
    #[napi]
    pub fn is_equal(&self, other: &Hash) -> bool {
        self.value == other.value
    }

    /// Bytes of the hash.
    #[napi]
    pub fn to_bytes(&self) -> Vec<u8> {
        let h: iroh::blobs::Hash = self.value.parse().unwrap();
        h.as_bytes().to_vec()
    }

    /// Create a `Hash` from its raw bytes representation.
    #[napi(factory)]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let bytes: [u8; 32] = bytes.try_into().map_err(|b: Vec<u8>| {
            anyhow::anyhow!("expected byte array of length 32, got {}", b.len())
        })?;

        Ok(Hash {
            value: iroh::blobs::Hash::from_bytes(bytes).to_string(),
        })
    }

    /// Make a Hash from base32 or hex string
    #[napi(factory)]
    pub fn from_string(s: String) -> Result<Self> {
        let key = iroh::blobs::Hash::from_str(&s).map_err(anyhow::Error::from)?;
        Ok(key.into())
    }

    /// Convert the hash to a hex string.
    #[napi]
    pub fn to_string(&self, target: Option<String>) -> String {
        match target {
            Some(target) => {
                if target == "hex" {
                    let key = iroh::blobs::Hash::from_str(&self.value).unwrap();
                    key.to_hex()
                } else {
                    panic!("invalid target: {}", target);
                }
            }
            None => self.value.clone(),
        }
    }
}

impl From<&Hash> for iroh::blobs::Hash {
    fn from(value: &Hash) -> Self {
        value.value.parse().unwrap()
    }
}

/// An AddProgress event indicating an item was found with name `name`, that can be referred to by `id`
#[derive(Debug, Clone)]
#[napi(object)]
pub struct AddProgressFound {
    /// A new unique id for this entry.
    pub id: BigInt,
    /// The name of the entry.
    pub name: String,
    /// The size of the entry in bytes.
    pub size: BigInt,
}

/// An AddProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct AddProgressProgress {
    /// The unique id of the entry.
    pub id: BigInt,
    /// The offset of the progress, in bytes.
    pub offset: BigInt,
}

/// An AddProgress event indicated we are done with `id` and now have a hash `hash`
#[derive(Debug, Clone)]
#[napi(object)]
pub struct AddProgressDone {
    /// The unique id of the entry.
    pub id: BigInt,
    /// The hash of the entry.
    pub hash: String,
}

/// An AddProgress event indicating we are done with the the whole operation
#[derive(Debug, Clone)]
#[napi(object)]
pub struct AddProgressAllDone {
    /// The hash of the created data.
    pub hash: String,
    /// The format of the added data.
    pub format: BlobFormat,
    /// The tag of the added data.
    pub tag: Vec<u8>,
}

/// Progress updates for the add operation.
#[derive(Debug, Clone, Default)]
#[napi(object)]
pub struct AddProgress {
    /// An item was found with name `name`, from now on referred to via `id`
    pub found: Option<AddProgressFound>,
    /// We got progress ingesting item `id`.
    pub progress: Option<AddProgressProgress>,
    /// We are done with `id`, and the hash is `hash`.
    pub done: Option<AddProgressDone>,
    /// We are done with the whole operation.
    pub all_done: Option<AddProgressAllDone>,
}

impl AddProgress {
    fn convert(value: anyhow::Result<iroh::blobs::provider::AddProgress>) -> Result<Self> {
        match value {
            Ok(value) => match value {
                iroh::blobs::provider::AddProgress::Found { id, name, size } => Ok(AddProgress {
                    found: Some(AddProgressFound {
                        id: id.into(),
                        name,
                        size: size.into(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::provider::AddProgress::Progress { id, offset } => Ok(AddProgress {
                    progress: Some(AddProgressProgress {
                        id: id.into(),
                        offset: offset.into(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::provider::AddProgress::Done { id, hash } => Ok(AddProgress {
                    done: Some(AddProgressDone {
                        id: id.into(),
                        hash: hash.to_string(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::provider::AddProgress::AllDone { hash, format, tag } => {
                    Ok(AddProgress {
                        all_done: Some(AddProgressAllDone {
                            hash: hash.to_string(),
                            format: format.into(),
                            tag: tag.0.to_vec(),
                        }),
                        ..Default::default()
                    })
                }
                iroh::blobs::provider::AddProgress::Abort(err) => {
                    Err(anyhow::Error::from(err).into())
                }
            },
            Err(err) => Err(err.into()),
        }
    }
}

/// A format identifier
#[derive(Debug, PartialEq, Eq)]
#[napi(string_enum)]
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
#[derive(Debug)]
#[napi]
pub struct BlobDownloadOptions(iroh::client::blobs::DownloadOptions);

#[napi]
impl BlobDownloadOptions {
    /// Create a BlobDownloadRequest
    #[napi(constructor)]
    pub fn new(format: BlobFormat, nodes: Vec<NodeAddr>, tag: &SetTagOption) -> Result<Self> {
        let nodes = nodes
            .into_iter()
            .map(|node| node.try_into())
            .collect::<std::result::Result<_, _>>()?;
        Ok(BlobDownloadOptions(iroh::client::blobs::DownloadOptions {
            format: format.into(),
            nodes,
            tag: tag.into(),
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
#[derive(Debug)]
#[napi(string_enum)]
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
#[derive(Debug)]
#[napi(string_enum)]
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

/// A DownloadProgress event indicating an item was found with hash `hash`, that can be referred to by `id`
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressFound {
    /// A new unique id for this entry.
    pub id: BigInt,
    /// child offset
    pub child: BigInt,
    /// The hash of the entry.
    pub hash: String,
    /// The size of the entry in bytes.
    pub size: BigInt,
}

/// A DownloadProgress event indicating an entry was found locally
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressFoundLocal {
    /// child offset
    pub child: BigInt,
    /// The hash of the entry.
    pub hash: String,
    /// The size of the entry in bytes.
    pub size: BigInt,
    // TODO:
    // /// The ranges that are available locally.
    // pub valid_ranges: RangeSpec,
}

/// A DownloadProgress event indicating an item was found with hash `hash`, that can be referred to by `id`
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressFoundHashSeq {
    /// Number of children in the collection, if known.
    pub children: BigInt,
    /// The hash of the entry.
    pub hash: String,
}

/// A DownloadProgress event indicating we got progress ingesting item `id`.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressProgress {
    /// The unique id of the entry.
    pub id: BigInt,
    /// The offset of the progress, in bytes.
    pub offset: BigInt,
}

/// A DownloadProgress event indicated we are done with `id`
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressDone {
    /// The unique id of the entry.
    pub id: BigInt,
}

/// A DownloadProgress event indicating we are done with the whole operation
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressAllDone {
    /// The number of bytes written
    pub bytes_written: BigInt,
    /// The number of bytes read
    pub bytes_read: BigInt,
    /// The time it took to transfer the data, in milliseconds.
    pub elapsed: BigInt,
}

/// A DownloadProgress event indicating we got an error and need to abort
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DownloadProgressAbort {
    pub error: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
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
#[derive(Debug, Clone, Default)]
#[napi(object)]
pub struct DownloadProgress {
    /// Initial state if subscribing to a running or queued transfer.
    pub initial_state: Option<DownloadProgressInitialState>,
    /// A new connection was established.
    pub connected: Option<()>,
    /// An item was found with hash `hash`, from now on referred to via `id`
    pub found: Option<DownloadProgressFound>,
    /// Data was found locally
    pub found_local: Option<DownloadProgressFoundLocal>,
    /// An item was found with hash `hash`, from now on referred to via `id`
    pub found_hash_seq: Option<DownloadProgressFoundHashSeq>,
    /// We got progress ingesting item `id`.
    pub progress: Option<DownloadProgressProgress>,
    /// We are done with `id`, and the hash is `hash`.
    pub done: Option<DownloadProgressDone>,
    /// We are done with the whole operation.
    pub all_done: Option<DownloadProgressAllDone>,
}

impl DownloadProgress {
    fn convert(value: anyhow::Result<iroh::blobs::get::db::DownloadProgress>) -> Result<Self> {
        match value {
            Ok(value) => match value {
                iroh::blobs::get::db::DownloadProgress::InitialState(transfer_state) => {
                    Ok(DownloadProgress {
                        initial_state: Some(DownloadProgressInitialState {
                            connected: transfer_state.connected,
                        }),
                        ..Default::default()
                    })
                }
                iroh::blobs::get::db::DownloadProgress::FoundLocal {
                    child, hash, size, ..
                } => Ok(DownloadProgress {
                    found_local: Some(DownloadProgressFoundLocal {
                        child: u64::from(child).into(),
                        hash: hash.to_string(),
                        size: size.value().into(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::get::db::DownloadProgress::Connected => Ok(DownloadProgress {
                    connected: Some(()),
                    ..Default::default()
                }),
                iroh::blobs::get::db::DownloadProgress::Found {
                    id,
                    hash,
                    child,
                    size,
                } => Ok(DownloadProgress {
                    found: Some(DownloadProgressFound {
                        id: id.into(),
                        hash: hash.to_string(),
                        child: u64::from(child).into(),
                        size: size.into(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::get::db::DownloadProgress::FoundHashSeq { hash, children } => {
                    Ok(DownloadProgress {
                        found_hash_seq: Some(DownloadProgressFoundHashSeq {
                            hash: hash.to_string(),
                            children: children.into(),
                        }),
                        ..Default::default()
                    })
                }
                iroh::blobs::get::db::DownloadProgress::Progress { id, offset } => {
                    Ok(DownloadProgress {
                        progress: Some(DownloadProgressProgress {
                            id: id.into(),
                            offset: offset.into(),
                        }),
                        ..Default::default()
                    })
                }
                iroh::blobs::get::db::DownloadProgress::Done { id } => Ok(DownloadProgress {
                    done: Some(DownloadProgressDone { id: id.into() }),
                    ..Default::default()
                }),
                iroh::blobs::get::db::DownloadProgress::AllDone(stats) => Ok(DownloadProgress {
                    all_done: Some(DownloadProgressAllDone {
                        bytes_written: stats.bytes_written.into(),
                        bytes_read: stats.bytes_read.into(),
                        elapsed: stats.elapsed.as_millis().into(),
                    }),
                    ..Default::default()
                }),
                iroh::blobs::get::db::DownloadProgress::Abort(err) => {
                    Err(anyhow::Error::from(err).into())
                }
            },
            Err(err) => Err(err.into()),
        }
    }
}

/// A chunk range specification as a sequence of chunk offsets
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct RangeSpec(pub(crate) iroh::blobs::protocol::RangeSpec);

#[napi]
impl RangeSpec {
    /// Checks if this [`RangeSpec`] does not select any chunks in the blob
    #[napi]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if this [`RangeSpec`] selects all chunks in the blob
    #[napi]
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
#[napi(object)]
pub struct BlobInfo {
    /// Location of the blob
    pub path: String,
    /// The hash of the blob
    pub hash: String,
    /// The size of the blob
    pub size: BigInt,
}

impl From<iroh::client::blobs::BlobInfo> for BlobInfo {
    fn from(value: iroh::client::blobs::BlobInfo) -> Self {
        BlobInfo {
            path: value.path,
            hash: value.hash.to_string(),
            size: value.size.into(),
        }
    }
}

/// A response to a list blobs request
#[derive(Debug, Clone)]
#[napi(object)]
pub struct IncompleteBlobInfo {
    /// The size we got
    pub size: BigInt,
    /// The size we expect
    pub expected_size: BigInt,
    /// The hash of the blob
    pub hash: String,
}

impl From<iroh::client::blobs::IncompleteBlobInfo> for IncompleteBlobInfo {
    fn from(value: iroh::client::blobs::IncompleteBlobInfo) -> Self {
        IncompleteBlobInfo {
            size: value.size.into(),
            expected_size: value.expected_size.into(),
            hash: value.hash.to_string(),
        }
    }
}

/// A response to a list collections request
#[derive(Debug, Clone)]
#[napi(object)]
pub struct CollectionInfo {
    /// Tag of the collection
    pub tag: Vec<u8>,
    /// Hash of the collection
    pub hash: String,
    /// Number of children in the collection
    ///
    /// This is an optional field, because the data is not always available.
    pub total_blobs_count: Option<BigInt>,
    /// Total size of the raw data referred to by all links
    ///
    /// This is an optional field, because the data is not always available.
    pub total_blobs_size: Option<BigInt>,
}

impl From<iroh::client::blobs::CollectionInfo> for CollectionInfo {
    fn from(value: iroh::client::blobs::CollectionInfo) -> Self {
        CollectionInfo {
            tag: value.tag.0.to_vec(),
            hash: value.hash.to_string(),
            total_blobs_count: value.total_blobs_count.map(Into::into),
            total_blobs_size: value.total_blobs_size.map(Into::into),
        }
    }
}

/// A collection of blobs
#[derive(Debug)]
#[napi]
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

#[napi]
impl Collection {
    /// Create a new empty collection
    #[allow(clippy::new_without_default)]
    #[napi(constructor)]
    pub fn new() -> Self {
        Collection(RwLock::new(
            iroh::blobs::format::collection::Collection::default(),
        ))
    }

    /// Add the given blob to the collection
    #[napi]
    pub fn push(&self, name: String, hash: String) -> Result<()> {
        let hash = hash.parse().map_err(anyhow::Error::from)?;
        self.0.write().unwrap().push(name, hash);
        Ok(())
    }

    /// Check if the collection is empty
    #[napi]
    pub fn is_empty(&self) -> bool {
        self.0.read().unwrap().is_empty()
    }

    /// Get the names of the blobs in this collection
    #[napi]
    pub fn names(&self) -> Vec<String> {
        let res = self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        res
    }

    /// Get the links to the blobs in this collection
    #[napi]
    pub fn links(&self) -> Vec<String> {
        let res = self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(_, hash)| hash.to_string())
            .collect();
        res
    }

    /// Get the blobs associated with this collection
    #[napi]
    pub fn blobs(&self) -> Vec<LinkAndName> {
        let res = self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(name, hash)| LinkAndName {
                name: name.clone(),
                link: hash.to_string(),
            })
            .collect();
        res
    }

    /// Returns the number of blobs in this collection
    #[napi]
    pub fn length(&self) -> BigInt {
        let res = self.0.read().unwrap().len() as u64;
        res.into()
    }
}

/// `LinkAndName` includes a name and a hash for a blob in a collection
#[derive(Clone, Debug)]
#[napi(object)]
pub struct LinkAndName {
    /// The name associated with this [`Hash`]
    pub name: String,
    /// The [`Hash`] of the blob
    pub link: String,
}

/// Events emitted by the provider informing about the current status.
#[derive(Clone, Debug, Default)]
#[napi(object)]
pub struct BlobProvideEvent {
    /// A new collection or tagged blob has been added
    pub tagged_blob_added: Option<TaggedBlobAdded>,
    /// A new client connected to the node.
    pub client_connected: Option<ClientConnected>,
    /// A request was received from a client.
    pub get_request_received: Option<GetRequestReceived>,
    /// A sequence of hashes has been found and is being transferred.
    pub transfer_hash_seq_started: Option<TransferHashSeqStarted>,
    /// A chunk of a blob was transferred.
    ///
    /// These events will be sent with try_send, so you can not assume that you
    /// will receive all of them.
    pub transfer_progress: Option<TransferProgress>,
    /// A blob in a sequence was transferred.
    pub transfer_blob_completed: Option<TransferBlobCompleted>,
    /// A request was completed and the data was sent to the client.
    pub transfer_completed: Option<TransferCompleted>,
    /// A request was aborted because the client disconnected.
    pub transfer_aborted: Option<TransferAborted>,
}

/// An BlobProvide event indicating a new tagged blob or collection was added
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TaggedBlobAdded {
    /// The hash of the added data
    pub hash: String,
    /// The format of the added data
    pub format: BlobFormat,
    /// The tag of the added data
    pub tag: Vec<u8>,
}

/// A new client connected to the node.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ClientConnected {
    /// An unique connection id.
    pub connection_id: BigInt,
}

/// A request was received from a client.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct GetRequestReceived {
    /// An unique connection id.
    pub connection_id: BigInt,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: BigInt,
    /// The hash for which the client wants to receive data.
    pub hash: String,
}

/// A sequence of hashes has been found and is being transferred.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TransferHashSeqStarted {
    /// An unique connection id.
    pub connection_id: BigInt,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: BigInt,
    /// The number of blobs in the sequence.
    pub num_blobs: BigInt,
}

/// A chunk of a blob was transferred.
///
/// These events will be sent with try_send, so you can not assume that you
/// will receive all of them.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TransferProgress {
    /// An unique connection id.
    pub connection_id: BigInt,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: BigInt,
    /// The hash for which we are transferring data.
    pub hash: String,
    /// Offset up to which we have transferred data.
    pub end_offset: BigInt,
}

/// A blob in a sequence was transferred.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TransferBlobCompleted {
    /// An unique connection id.
    pub connection_id: BigInt,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: BigInt,
    /// The hash of the blob
    pub hash: String,
    /// The index of the blob in the sequence.
    pub index: BigInt,
    /// The size of the blob transferred.
    pub size: BigInt,
}

/// A request was completed and the data was sent to the client.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TransferCompleted {
    /// An unique connection id.
    pub connection_id: BigInt,
    /// An identifier uniquely identifying this transfer request.
    pub request_id: BigInt,
    /// statistics about the transfer
    pub stats: TransferStats,
}

/// A request was aborted because the client disconnected.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TransferAborted {
    /// The quic connection id.
    pub connection_id: BigInt,
    /// An identifier uniquely identifying this request.
    pub request_id: BigInt,
    /// statistics about the transfer. This is None if the transfer
    /// was aborted before any data was sent.
    pub stats: Option<TransferStats>,
}

/// The stats for a transfer of a collection or blob.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct TransferStats {
    /// The total duration of the transfer in milliseconds.
    pub duration: BigInt,
}

impl BlobProvideEvent {
    pub(crate) fn convert(value: iroh::blobs::provider::Event) -> Result<Self> {
        match value {
            iroh::blobs::provider::Event::TaggedBlobAdded { hash, format, tag } => {
                Ok(BlobProvideEvent {
                    tagged_blob_added: Some(TaggedBlobAdded {
                        hash: hash.to_string(),
                        format: format.into(),
                        tag: tag.0.as_ref().to_vec(),
                    }),
                    ..Default::default()
                })
            }
            iroh::blobs::provider::Event::ClientConnected { connection_id } => {
                Ok(BlobProvideEvent {
                    client_connected: Some(ClientConnected {
                        connection_id: connection_id.into(),
                    }),
                    ..Default::default()
                })
            }
            iroh::blobs::provider::Event::GetRequestReceived {
                connection_id,
                request_id,
                hash,
            } => Ok(BlobProvideEvent {
                get_request_received: Some(GetRequestReceived {
                    connection_id: connection_id.into(),
                    request_id: request_id.into(),
                    hash: hash.to_string(),
                }),
                ..Default::default()
            }),
            iroh::blobs::provider::Event::TransferHashSeqStarted {
                connection_id,
                request_id,
                num_blobs,
            } => Ok(BlobProvideEvent {
                transfer_hash_seq_started: Some(TransferHashSeqStarted {
                    connection_id: connection_id.into(),
                    request_id: request_id.into(),
                    num_blobs: num_blobs.into(),
                }),
                ..Default::default()
            }),
            iroh::blobs::provider::Event::TransferProgress {
                connection_id,
                request_id,
                hash,
                end_offset,
            } => Ok(BlobProvideEvent {
                transfer_progress: Some(TransferProgress {
                    connection_id: connection_id.into(),
                    request_id: request_id.into(),
                    hash: hash.to_string(),
                    end_offset: end_offset.into(),
                }),
                ..Default::default()
            }),
            iroh::blobs::provider::Event::TransferBlobCompleted {
                connection_id,
                request_id,
                hash,
                index,
                size,
            } => Ok(BlobProvideEvent {
                transfer_blob_completed: Some(TransferBlobCompleted {
                    connection_id: connection_id.into(),
                    request_id: request_id.into(),
                    hash: hash.to_string(),
                    index: index.into(),
                    size: size.into(),
                }),
                ..Default::default()
            }),
            iroh::blobs::provider::Event::TransferCompleted {
                connection_id,
                request_id,
                stats,
            } => Ok(BlobProvideEvent {
                transfer_completed: Some(TransferCompleted {
                    connection_id: connection_id.into(),
                    request_id: request_id.into(),
                    stats: stats.as_ref().into(),
                }),
                ..Default::default()
            }),
            iroh::blobs::provider::Event::TransferAborted {
                connection_id,
                request_id,
                stats,
            } => Ok(BlobProvideEvent {
                transfer_aborted: Some(TransferAborted {
                    connection_id: connection_id.into(),
                    request_id: request_id.into(),
                    stats: stats.map(|s| s.as_ref().into()),
                }),
                ..Default::default()
            }),
        }
    }
}

impl From<&iroh::blobs::provider::TransferStats> for TransferStats {
    fn from(value: &iroh::blobs::provider::TransferStats) -> Self {
        Self {
            duration: value.duration.as_millis().into(),
        }
    }
}
