use std::str::FromStr;
use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};

use crate::node::IrohNode;
use crate::{block_on, IrohError, Tag};

impl IrohNode {
    pub fn blob_list_blobs(&self) -> Result<Vec<Arc<Hash>>, IrohError> {
        block_on(&self.async_runtime, async {
            let response = self
                .sync_client
                .blobs
                .list()
                .await
                .map_err(IrohError::blob)?;

            let hashes: Vec<Arc<Hash>> = response
                .map_ok(|i| Arc::new(Hash(i.hash)))
                .map_err(IrohError::blob)
                .try_collect()
                .await?;

            Ok(hashes)
        })
    }

    pub fn blob_get(&self, hash: Arc<Hash>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.async_runtime, async {
            let mut r = self
                .sync_client
                .blobs
                .read(hash.0)
                .await
                .map_err(IrohError::blob)?;
            let data = r.read_to_bytes().await.map_err(IrohError::blob)?;
            Ok(data.into())
        })
    }
    /// Get the size information on a single blob.
    ///
    /// Method only exist in FFI
    pub fn blob_size(&self, hash: Arc<Hash>) -> Result<u64, IrohError> {
        block_on(&self.async_runtime, async {
            let r = self
                .sync_client
                .blobs
                .read(hash.0)
                .await
                .map_err(IrohError::blob)?;
            Ok(r.size())
        })
    }

    /// Read all bytes of single blob.
    ///
    /// This allocates a buffer for the full blob. Use only if you know that the blob you're
    /// reading is small. If not sure, use [`Self::blobs_size`] and check the size with
    /// before calling [`Self::blobs_read_to_bytes`].
    pub fn blob_read_to_bytes(&self, hash: Arc<Hash>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .blobs
                .read_to_bytes(hash.0)
                .await
                .map(|b| b.to_vec())
                .map_err(IrohError::blob)
        })
    }

    /// Import a blob from a filesystem path.
    ///
    /// `path` should be an absolute path valid for the file system on which
    /// the node runs.
    /// If `in_place` is true, Iroh will assume that the data will not change and will share it in
    /// place without copying to the Iroh data directory.
    pub fn blob_add_from_path(
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
                .map_err(IrohError::blob)?;
            while let Some(progress) = stream.next().await {
                let progress = progress.map_err(IrohError::blob)?;
                if let Err(e) = cb.progress(Arc::new(progress.into())) {
                    return Err(e);
                }
            }
            Ok(())
        })
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

impl From<iroh::bytes::util::BlobFormat> for BlobFormat {
    fn from(value: iroh::bytes::util::BlobFormat) -> Self {
        match value {
            iroh::bytes::util::BlobFormat::Raw => BlobFormat::Raw,
            iroh::bytes::util::BlobFormat::HashSeq => BlobFormat::HashSeq,
        }
    }
}
