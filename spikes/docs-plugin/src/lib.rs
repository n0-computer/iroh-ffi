//! SPIKE: docs plugin — depends on the blobs AND gossip plugins as well as core.
use std::sync::Arc;

uniffi::setup_scaffolding!("iroh_ffi_docs");

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DocsError {
    #[error("docs failed: {reason}")]
    Failed { reason: String },
}

fn err(e: impl std::fmt::Display) -> DocsError {
    DocsError::Failed { reason: e.to_string() }
}

/// The docs protocol.
#[derive(uniffi::Object)]
pub struct Docs {
    inner: iroh_docs::protocol::Docs,
}

#[uniffi::export]
impl Docs {
    /// THE LOAD-BEARING CONSTRUCTOR: external types from THREE separate namespaces —
    /// `iroh_ffi::Endpoint`, `iroh_ffi_blobs::Store`, `iroh_ffi_gossip::Gossip`.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn memory(
        endpoint: Arc<iroh_ffi::Endpoint>,
        blobs: Arc<iroh_ffi_blobs::Store>,
        gossip: Arc<iroh_ffi_gossip::Gossip>,
    ) -> Result<Arc<Self>, DocsError> {
        let inner = iroh_docs::protocol::Docs::memory()
            .spawn(endpoint.raw().clone(), blobs.raw().clone(), gossip.raw().clone())
            .await
            .map_err(err)?;
        Ok(Arc::new(Self { inner }))
    }

    /// Create an author, proving the composed engine actually runs.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_create(&self) -> Result<String, DocsError> {
        let id = self.inner.api().author_create().await.map_err(err)?;
        Ok(id.to_string())
    }

    /// Create a document and return its namespace id.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn doc_create(&self) -> Result<String, DocsError> {
        let doc = self.inner.api().create().await.map_err(err)?;
        Ok(doc.id().to_string())
    }
}
