//! The thing we are trying to make possible: an FFI wrapper for `iroh-ping`
//! that lives in its own crate, its own uniffi namespace, and (under Design B)
//! its own shipped native library.
//!
//! Note what this crate does *not* do: it does not spawn a router, and it does
//! not own an endpoint. It exports `Ping` and implements `ProtocolCreator`, so
//! the consumer assembles it — in Python, JS, Kotlin or Swift — by passing it to
//! `Endpoint.bind(EndpointOptions(protocols={...}))`. That is the same
//! composition path a foreign-language protocol implementation would use.

use std::{str::FromStr, sync::Arc};

use iroh::{EndpointAddr, EndpointId};
use p3_iroh_core::{CallbackError, Connection, CoreError, Endpoint, ProtocolCreator};

uniffi::setup_scaffolding!("p3_iroh_ping_ffi");

/// The ping protocol. Register it with `EndpointOptions.protocols` under
/// [`Ping::alpn`], then use [`Ping::ping`] to ping someone else.
#[derive(Debug, uniffi::Object)]
pub struct Ping(iroh_ping::Ping);

#[uniffi::export]
impl Ping {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self(iroh_ping::Ping::new()))
    }

    /// The ALPN this protocol is registered under.
    pub fn alpn(&self) -> Vec<u8> {
        iroh_ping::ALPN.to_vec()
    }

    /// Ping `endpoint_id` at `direct_addrs`, returning the round trip in
    /// microseconds.
    ///
    /// `endpoint` was constructed by the core library; `raw()` gets us back to
    /// the `iroh::Endpoint` that `iroh_ping` wants.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn ping(
        &self,
        endpoint: Arc<Endpoint>,
        endpoint_id: String,
        direct_addrs: Vec<String>,
    ) -> Result<u64, CoreError> {
        let id = EndpointId::from_str(&endpoint_id).map_err(CoreError::new)?;
        let mut addr = EndpointAddr::new(id);
        for a in direct_addrs {
            addr = addr.with_ip_addr(a.parse::<std::net::SocketAddr>().map_err(CoreError::new)?);
        }
        let rtt = self
            .0
            .ping(endpoint.raw(), addr)
            .await
            .map_err(CoreError::new)?;
        Ok(rtt.as_micros() as u64)
    }

    /// Count of pings answered by this protocol instance.
    pub fn pings_received(&self) -> u64 {
        self.0.metrics().pings_recv.get()
    }

    /// Count of pings sent by this protocol instance.
    pub fn pings_sent(&self) -> u64 {
        self.0.metrics().pings_sent.get()
    }
}

/// This is what makes `Ping` droppable into `EndpointOptions.protocols`.
///
/// `Ping` is both the creator and the handler: the metrics live on the single
/// instance the consumer holds, so `pings_received()` observes what the router
/// answered.
#[uniffi::export]
impl ProtocolCreator for Ping {
    fn create(&self, _endpoint: Arc<Endpoint>) -> Arc<dyn p3_iroh_core::ProtocolHandler> {
        Arc::new(PingHandler(self.0.clone()))
    }
}

#[derive(Debug)]
struct PingHandler(iroh_ping::Ping);

#[async_trait::async_trait]
impl p3_iroh_core::ProtocolHandler for PingHandler {
    async fn accept(&self, conn: Arc<Connection>) -> Result<(), CallbackError> {
        // The router handed us the core library's `Connection` wrapper; `raw()`
        // gets the `iroh::endpoint::Connection` that `iroh_ping` accepts.
        iroh::protocol::ProtocolHandler::accept(&self.0, conn.raw().clone())
            .await
            .map_err(|e| CallbackError::Failed(e.to_string()))
    }

    async fn shutdown(&self) {}
}
