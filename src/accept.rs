//! Raw accept loop types: [`Incoming`], [`Accepting`], [`Connecting`], and
//! related address enums.
//!
//! The router (`Endpoint::bind` with `EndpointOptions.protocols`) covers the
//! common case of "dispatch by ALPN", but exposing these types lets the FFI
//! caller run its own accept loop, inspect ALPN before completing the
//! handshake, refuse connections, etc.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{Connection, EndpointId, IrohError};

/// Which side of a connection we are.
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum Side {
    /// We initiated this connection.
    Client,
    /// We accepted this connection.
    Server,
}

impl From<iroh::endpoint::Side> for Side {
    fn from(s: iroh::endpoint::Side) -> Self {
        match s {
            iroh::endpoint::Side::Client => Side::Client,
            iroh::endpoint::Side::Server => Side::Server,
        }
    }
}

/// Where an incoming connection came from.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum IncomingAddr {
    /// A direct connection from an IP address (`ip:port` string).
    Ip { addr: String },
    /// A connection via a relay.
    Relay {
        url: String,
        endpoint_id: Arc<EndpointId>,
    },
    /// A custom-transport connection (rendered as its debug form).
    Custom { description: String },
}

impl From<iroh::endpoint::IncomingAddr> for IncomingAddr {
    fn from(addr: iroh::endpoint::IncomingAddr) -> Self {
        match addr {
            iroh::endpoint::IncomingAddr::Ip(socket) => IncomingAddr::Ip {
                addr: socket.to_string(),
            },
            iroh::endpoint::IncomingAddr::Relay { url, endpoint_id } => IncomingAddr::Relay {
                url: url.to_string(),
                endpoint_id: Arc::new(endpoint_id.into()),
            },
            iroh::endpoint::IncomingAddr::Custom(custom) => IncomingAddr::Custom {
                description: format!("{custom:?}"),
            },
            _ => IncomingAddr::Custom {
                description: "unknown".into(),
            },
        }
    }
}

/// The local address that received an incoming connection.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum IncomingLocalAddr {
    /// Direct IP (`ip` string if available).
    Ip { addr: Option<String> },
    /// Relay path.
    Relay { url: String },
    /// Custom transport.
    Custom { description: Option<String> },
}

impl From<iroh::endpoint::LocalTransportAddr> for IncomingLocalAddr {
    fn from(value: iroh::endpoint::LocalTransportAddr) -> Self {
        match value {
            iroh::endpoint::LocalTransportAddr::Ip(ip) => IncomingLocalAddr::Ip {
                addr: ip.map(|i| i.to_string()),
            },
            iroh::endpoint::LocalTransportAddr::Relay(url) => IncomingLocalAddr::Relay {
                url: url.to_string(),
            },
            iroh::endpoint::LocalTransportAddr::Custom(custom) => IncomingLocalAddr::Custom {
                description: custom.map(|c| format!("{c:?}")),
            },
            _ => IncomingLocalAddr::Custom { description: None },
        }
    }
}

/// An incoming connection that has not yet begun its server-side handshake.
///
/// Consume via [`Self::accept`] / [`Self::refuse`] / [`Self::retry`] / [`Self::ignore`].
/// Each `Incoming` can only be consumed once.
#[derive(uniffi::Object)]
pub struct Incoming(Mutex<Option<iroh::endpoint::Incoming>>);

impl Incoming {
    pub(crate) fn new(inner: iroh::endpoint::Incoming) -> Self {
        Self(Mutex::new(Some(inner)))
    }
}

#[uniffi::export]
impl Incoming {
    /// Begin the server-side handshake, producing an [`Accepting`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept(&self) -> Result<Accepting, IrohError> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        let accepting = inner.accept()?;
        Ok(Accepting::new(accepting))
    }

    /// Reject this incoming connection attempt.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn refuse(&self) -> Result<(), IrohError> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        inner.refuse();
        Ok(())
    }

    /// Respond with a retry packet, requiring the client to retry with address
    /// validation.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn retry(&self) -> Result<(), IrohError> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        inner
            .retry()
            .map_err(|e| anyhow::anyhow!("retry failed: {e:?}").into())
    }

    /// Drop this incoming connection without sending any reply.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn ignore(&self) -> Result<(), IrohError> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        inner.ignore();
        Ok(())
    }

    /// The local address that received this incoming connection.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn local_addr(&self) -> Result<IncomingLocalAddr, IrohError> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        Ok(inner.local_addr().into())
    }

    /// The remote address that originated this incoming connection.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_addr(&self) -> Result<IncomingAddr, IrohError> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        Ok(inner.remote_addr().into())
    }

    /// True if the remote address has been validated by the QUIC retry mechanism.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_addr_validated(&self) -> Result<bool, IrohError> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Incoming has already been consumed"))?;
        Ok(inner.remote_addr_validated())
    }
}

/// A server-side handshake in progress. Await with [`Self::connect`].
#[derive(uniffi::Object)]
pub struct Accepting(Mutex<Option<iroh::endpoint::Accepting>>);

impl Accepting {
    pub(crate) fn new(inner: iroh::endpoint::Accepting) -> Self {
        Self(Mutex::new(Some(inner)))
    }
}

#[uniffi::export]
impl Accepting {
    /// Wait for the handshake to complete, producing a [`Connection`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect(&self) -> Result<Connection, IrohError> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Accepting has already been consumed"))?;
        let conn = inner.await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(conn.into())
    }

    /// Read the ALPN protocol from the peer's handshake data (resolves once
    /// the ClientHello has been received).
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn alpn(&self) -> Result<Vec<u8>, IrohError> {
        let mut guard = self.0.lock().await;
        let inner = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Accepting has already been consumed"))?;
        Ok(inner.alpn().await?)
    }
}

/// A client-side handshake in progress. Await with [`Self::connect`].
#[derive(uniffi::Object)]
pub struct Connecting(Mutex<Option<iroh::endpoint::Connecting>>);

impl Connecting {
    pub(crate) fn new(inner: iroh::endpoint::Connecting) -> Self {
        Self(Mutex::new(Some(inner)))
    }
}

#[uniffi::export]
impl Connecting {
    /// Wait for the handshake to complete, producing a [`Connection`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect(&self) -> Result<Connection, IrohError> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Connecting has already been consumed"))?;
        let conn = inner.await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(conn.into())
    }

    /// Read the ALPN protocol from the peer's handshake data (resolves once
    /// the server has responded with its ServerHello).
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn alpn(&self) -> Result<Vec<u8>, IrohError> {
        let mut guard = self.0.lock().await;
        let inner = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Connecting has already been consumed"))?;
        Ok(inner.alpn().await?)
    }

    /// The [`EndpointId`] this connection attempt targets.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_id(&self) -> Result<Arc<EndpointId>, IrohError> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Connecting has already been consumed"))?;
        Ok(Arc::new(inner.remote_id().into()))
    }
}
