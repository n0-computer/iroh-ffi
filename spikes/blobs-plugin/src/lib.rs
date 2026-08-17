//! SPIKE: minimal blobs plugin. Enough surface to be a real dependency of the docs plugin.
use std::sync::Arc;

uniffi::setup_scaffolding!("iroh_ffi_blobs");

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BlobsError {
    #[error("blobs failed: {reason}")]
    Failed { reason: String },
}

fn err(e: impl std::fmt::Display) -> BlobsError {
    BlobsError::Failed { reason: e.to_string() }
}

/// An in-memory blob store.
#[derive(uniffi::Object)]
pub struct Store {
    inner: iroh_blobs::store::mem::MemStore,
}

#[uniffi::export]
impl Store {
    /// `async` because `MemStore::new()` spawns an actor task and so needs a tokio
    /// context — a sync uniffi constructor has none (cf. finding 21).
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn memory() -> Arc<Self> {
        Arc::new(Self { inner: iroh_blobs::store::mem::MemStore::new() })
    }

    /// Add bytes, returning the BLAKE3 hash as hex.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_bytes(&self, data: Vec<u8>) -> Result<String, BlobsError> {
        let tag = self.inner.add_bytes(data).await.map_err(err)?;
        Ok(tag.hash.to_string())
    }

    /// Read a blob back by hex hash.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn get_bytes(&self, hash: String) -> Result<Vec<u8>, BlobsError> {
        let hash: iroh_blobs::Hash = hash.parse().map_err(err)?;
        let bytes = self.inner.get_bytes(hash).await.map_err(err)?;
        Ok(bytes.to_vec())
    }
}

#[uniffi::export]
impl Store {
    /// Takes an EXTERNAL `Endpoint` from the core namespace, and — crucially — makes this
    /// crate actually *reference* `iroh-ffi`. Without a real reference the linker drops the
    /// core dylib and this plugin statically embeds its own copy of iroh (167 MB vs 7 MB).
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn serve(&self, endpoint: Arc<iroh_ffi::Endpoint>) -> Result<(), BlobsError> {
        let _router = iroh::protocol::Router::builder(endpoint.raw().clone())
            .accept(iroh_blobs::ALPN, iroh_blobs::BlobsProtocol::new(&self.inner, None))
            .spawn();
        Ok(())
    }
}

impl Store {
    /// Escape hatch for downstream plugins (docs needs the real `iroh_blobs::api::Store`).
    pub fn raw(&self) -> &iroh_blobs::api::Store {
        &self.inner
    }
}
