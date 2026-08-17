//! SPIKE: stand-in for an independently maintained, independently published
//! `iroh-ping-ffi` plugin.
//!
//! What this is testing:
//!
//! 1. **Linkage.** This crate depends on `iroh-ffi`, which is buildable only as
//!    staticlib/cdylib/dylib (no rlib), so `iroh` must resolve to the single copy inside
//!    `libiroh_ffi.dylib` rather than being statically linked again here.
//! 2. **uniffi external types.** `Endpoint` and `EndpointAddr` are `uniffi::Object`s owned
//!    by the `iroh_ffi` namespace. Accepting them here means an `Arc` handle minted by the
//!    core library is lifted by this library — only sound if there is one monomorphization.
//! 3. **A Rust impl of a foreign-exportable trait.** `PingHandler` implements
//!    `iroh_ffi::ProtocolHandler`, mirroring how `src/services.rs` wraps
//!    `iroh_services::IrohServicesPreset` behind the `Preset` trait.

use std::sync::Arc;

use iroh_ffi::{CallbackError, Connection, Endpoint, EndpointAddr, ProtocolHandler};

uniffi::setup_scaffolding!("iroh_ffi_ping");

/// Errors from the ping protocol.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PingError {
    #[error("ping failed: {message}")]
    Failed { message: String },
}

/// The ALPN this protocol accepts on.
#[uniffi::export]
pub fn alpn() -> Vec<u8> {
    iroh_ping::ALPN.to_vec()
}

/// Ping protocol state.
#[derive(Debug, uniffi::Object)]
pub struct Ping {
    inner: iroh_ping::Ping,
}

#[uniffi::export]
impl Ping {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: iroh_ping::Ping::new(),
        })
    }

    /// Ping `addr` over `endpoint`, returning the round-trip time in milliseconds.
    ///
    /// Both arguments are **external types** owned by the `iroh_ffi` namespace — this is
    /// the load-bearing call for the spike.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn ping(
        &self,
        endpoint: Arc<Endpoint>,
        addr: Arc<EndpointAddr>,
    ) -> Result<u64, PingError> {
        let iroh_addr: iroh::EndpointAddr =
            (*addr).clone().try_into().map_err(|e: iroh_ffi::IrohError| {
                PingError::Failed {
                    message: e.message(),
                }
            })?;

        let rtt = self
            .inner
            .ping(endpoint.raw(), iroh_addr)
            .await
            .map_err(|e| PingError::Failed {
                message: format!("{e:#}"),
            })?;

        Ok(rtt.as_millis() as u64)
    }

    /// A handler that can be registered on the core's router for [`alpn`].
    pub fn handler(&self) -> Arc<dyn ProtocolHandler> {
        Arc::new(PingHandler {
            inner: self.inner.clone(),
        })
    }
}

/// Rust implementation of the core's foreign-exportable `ProtocolHandler` trait.
#[derive(Debug)]
struct PingHandler {
    inner: iroh_ping::Ping,
}

#[async_trait::async_trait]
impl ProtocolHandler for PingHandler {
    async fn accept(&self, conn: Arc<Connection>) -> Result<(), CallbackError> {
        iroh::protocol::ProtocolHandler::accept(&self.inner, conn.raw().clone())
            .await
            .map_err(|_| CallbackError::Error)?;
        Ok(())
    }

    async fn shutdown(&self) {}
}
