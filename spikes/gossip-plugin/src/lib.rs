//! SPIKE: minimal gossip plugin — the second dependency of the docs plugin.
use std::sync::Arc;

uniffi::setup_scaffolding!("iroh_ffi_gossip");

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GossipError {
    #[error("gossip failed: {reason}")]
    Failed { reason: String },
}

/// A spawned gossip protocol instance.
#[derive(uniffi::Object)]
pub struct Gossip {
    inner: iroh_gossip::net::Gossip,
}

#[uniffi::export]
impl Gossip {
    /// Takes an EXTERNAL `Endpoint` from the core namespace.
    ///
    /// `async` even though `Gossip::builder().spawn()` is sync: it spawns background tasks
    /// and so needs a tokio context (cf. finding 21).
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn spawn(endpoint: Arc<iroh_ffi::Endpoint>) -> Arc<Self> {
        let inner = iroh_gossip::net::Gossip::builder().spawn(endpoint.raw().clone());
        Arc::new(Self { inner })
    }

    #[uniffi::method]
    pub fn max_message_size(&self) -> u64 {
        self.inner.max_message_size() as u64
    }
}

impl Gossip {
    /// Escape hatch for downstream plugins.
    pub fn raw(&self) -> &iroh_gossip::net::Gossip {
        &self.inner
    }
}
