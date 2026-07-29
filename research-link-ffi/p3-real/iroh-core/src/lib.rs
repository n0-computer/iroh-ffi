//! Minimal stand-in for `iroh-ffi`.
//!
//! Deliberately mirrors the real crate's protocol-registration shape: there is
//! no exported `Router` object. Protocols are registered by passing
//! [`ProtocolCreator`]s in [`EndpointOptions::protocols`], and `Endpoint::bind`
//! spawns the router internally. See `iroh-ffi/src/endpoint.rs`.

use std::{collections::HashMap, sync::Arc};

uniffi::setup_scaffolding!("p3_iroh_core");

/// A stringly error, to keep the prototype's surface small.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum CoreError {
    #[error("{0}")]
    Failed(String),
}

impl CoreError {
    pub fn new(e: impl std::fmt::Display) -> Self {
        Self::Failed(e.to_string())
    }
}

/// Errors crossing back from a foreign (or downstream-crate) protocol handler.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum CallbackError {
    #[error("{0}")]
    Failed(String),
}

impl From<uniffi::UnexpectedUniFFICallbackError> for CallbackError {
    fn from(e: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::Failed(e.to_string())
    }
}

/// Builds the protocol handler for an endpoint, once that endpoint exists.
///
/// `with_foreign` means both foreign languages *and* other Rust crates can
/// implement this — which is what lets `p3-iroh-ping-ffi` register itself
/// without this crate knowing anything about ping.
#[uniffi::export(with_foreign)]
pub trait ProtocolCreator: std::fmt::Debug + Send + Sync + 'static {
    fn create(&self, endpoint: Arc<Endpoint>) -> Arc<dyn ProtocolHandler>;
}

/// Handles one accepted connection.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait ProtocolHandler: Send + Sync + 'static {
    async fn accept(&self, conn: Arc<Connection>) -> Result<(), CallbackError>;
    async fn shutdown(&self);
}

#[derive(derive_more::Debug, Clone)]
struct ProtocolWrapper {
    #[debug("handler")]
    handler: Arc<dyn ProtocolHandler>,
}

impl iroh::protocol::ProtocolHandler for ProtocolWrapper {
    async fn accept(
        &self,
        conn: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        self.handler
            .accept(Arc::new(Connection(conn)))
            .await
            .map_err(iroh::protocol::AcceptError::from_err)?;
        Ok(())
    }

    async fn shutdown(&self) {
        self.handler.shutdown().await;
    }
}

/// Options for [`Endpoint::bind`].
#[derive(Debug, uniffi::Record)]
pub struct EndpointOptions {
    /// Protocols to accept, keyed by ALPN. If non-empty, an internal router is
    /// spawned to dispatch incoming connections.
    #[uniffi(default = None)]
    pub protocols: Option<HashMap<Vec<u8>, Arc<dyn ProtocolCreator>>>,
}

/// The FFI wrapper around `iroh::Endpoint`.
#[derive(Clone, uniffi::Object)]
pub struct Endpoint {
    inner: iroh::Endpoint,
    router: Option<iroh::protocol::Router>,
}

#[uniffi::export]
impl Endpoint {
    /// Bind an endpoint. The `Minimal` preset means localhost, no relays and no
    /// discovery — enough for a same-process direct-address test.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn bind(options: EndpointOptions) -> Result<Self, CoreError> {
        let inner = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
            .bind()
            .await
            .map_err(CoreError::new)?;

        let router = match options.protocols {
            Some(protocols) if !protocols.is_empty() => {
                let mut builder = iroh::protocol::Router::builder(inner.clone());
                let wrapper = Arc::new(Endpoint {
                    inner: inner.clone(),
                    router: None,
                });
                for (alpn, creator) in protocols {
                    let handler = creator.create(wrapper.clone());
                    builder = builder.accept(alpn, ProtocolWrapper { handler });
                }
                Some(builder.spawn())
            }
            _ => None,
        };

        Ok(Self { inner, router })
    }

    /// This endpoint's ID, hex-encoded.
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    /// Direct socket addresses this endpoint is bound to, as `ip:port` strings.
    pub fn direct_addrs(&self) -> Vec<String> {
        self.inner
            .bound_sockets()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Address of a process-global static inside this crate. Two libraries that
    /// share one copy of `p3-iroh-core` report the same address.
    pub fn core_static_addr(&self) -> u64 {
        static MARKER: u8 = 0;
        &MARKER as *const u8 as u64
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn close(&self) -> Result<(), CoreError> {
        match &self.router {
            Some(router) => router.shutdown().await.map_err(CoreError::new)?,
            None => self.inner.close().await,
        }
        Ok(())
    }
}

impl Endpoint {
    /// The Rust-native accessor a protocol crate needs.
    ///
    /// In the real `iroh-ffi` this is `pub(crate) fn raw()`; splitting protocol
    /// wrappers into their own crates requires promoting it to `pub`.
    pub fn raw(&self) -> &iroh::Endpoint {
        &self.inner
    }
}

/// The FFI wrapper around `iroh::endpoint::Connection`.
#[derive(Clone, uniffi::Object)]
pub struct Connection(iroh::endpoint::Connection);

impl Connection {
    /// Same story as [`Endpoint::raw`] — must be `pub` for downstream protocol
    /// crates, because the router hands them an FFI `Connection` and they need
    /// the real one back.
    pub fn raw(&self) -> &iroh::endpoint::Connection {
        &self.0
    }
}

#[uniffi::export]
impl Connection {
    /// The remote endpoint's ID, hex-encoded.
    pub fn remote_id(&self) -> String {
        self.0.remote_id().to_string()
    }
}
