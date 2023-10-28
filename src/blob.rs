use std::sync::Arc;

use futures::TryStreamExt;

use crate::node::IrohNode;
use crate::{block_on, IrohError};

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
    pub fn blobs_size(&self, hash: Arc<Hash>) -> Result<u64, IrohError> {
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
