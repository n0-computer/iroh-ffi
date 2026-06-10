use std::{str::FromStr, sync::Arc};

use iroh::endpoint::{self, presets, presets::Preset as _};
use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;
use tokio::sync::Mutex;

use crate::{EndpointAddr, RelayMode, SecretKey, WatchHandle, path, watch};

/// A mutable handle to an endpoint builder.
///
/// Mirrors the uniffi `EndpointBuilder`. A "preset" in JS is simply any
/// function `(builder: EndpointBuilder) => void` — call one of the built-in
/// [`presetN0`] / [`presetMinimal`] / [`presetN0DisableRelay`] helpers (which
/// install the crypto provider) and then layer on your own configuration.
///
/// ```js
/// const b = Endpoint.builder()
/// presetMinimal(b)
/// b.alpns([alpn])
/// const ep = await b.bind()
/// ```
#[napi]
pub struct EndpointBuilder {
    inner: std::sync::Mutex<Option<iroh::endpoint::Builder>>,
}

impl EndpointBuilder {
    fn new(builder: iroh::endpoint::Builder) -> Self {
        Self {
            inner: std::sync::Mutex::new(Some(builder)),
        }
    }

    fn map<F>(&self, f: F)
    where
        F: FnOnce(iroh::endpoint::Builder) -> iroh::endpoint::Builder,
    {
        let mut guard = self.inner.lock().unwrap();
        let b = guard.take().expect("EndpointBuilder consumed");
        *guard = Some(f(b));
    }
}

#[napi]
impl EndpointBuilder {
    /// Replay the n0 production preset (relays + discovery + crypto provider).
    #[napi]
    pub fn apply_n0(&self) {
        self.map(|b| presets::N0.apply(b));
    }

    /// Replay the minimal preset (crypto provider only, no external deps).
    #[napi]
    pub fn apply_minimal(&self) {
        self.map(|b| presets::Minimal.apply(b));
    }

    /// Replay the n0 preset with relays disabled.
    #[napi]
    pub fn apply_n0_disable_relay(&self) {
        self.map(|b| presets::N0DisableRelay.apply(b));
    }

    /// Set the endpoint secret key (32 bytes).
    #[napi]
    pub fn secret_key(&self, bytes: Vec<u8>) -> Result<()> {
        let key: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("secret_key must be 32 bytes"))?;
        self.map(|b| b.secret_key(iroh::SecretKey::from_bytes(&key)));
        Ok(())
    }

    /// Set the advertised ALPNs.
    #[napi]
    pub fn alpns(&self, alpns: Vec<Vec<u8>>) {
        self.map(|b| b.alpns(alpns));
    }

    /// Set the relay mode.
    #[napi]
    pub fn relay_mode(&self, mode: &RelayMode) {
        let mode = mode.0.clone();
        self.map(|b| b.relay_mode(mode));
    }

    /// Set the address the endpoint binds to (`host:port`).
    #[napi]
    pub fn bind_addr(&self, addr: String) -> Result<()> {
        let socket = std::net::SocketAddr::from_str(&addr).map_err(anyhow::Error::from)?;
        let mut guard = self.inner.lock().unwrap();
        let b = guard.take().expect("EndpointBuilder consumed");
        *guard = Some(b.bind_addr(socket).map_err(anyhow::Error::from)?);
        Ok(())
    }

    /// Bind the endpoint.
    #[napi]
    pub async fn bind(&self) -> Result<Endpoint> {
        let builder = self
            .inner
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| anyhow::anyhow!("EndpointBuilder already consumed"))?;
        let endpoint = builder.bind().await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Endpoint { inner: endpoint })
    }
}

/// The n0 production preset (relays + discovery).
#[napi]
pub fn preset_n0(builder: &EndpointBuilder) {
    builder.apply_n0();
}

/// The minimal preset (no external dependencies; good for tests / offline).
#[napi]
pub fn preset_minimal(builder: &EndpointBuilder) {
    builder.apply_minimal();
}

/// The n0 preset with relays disabled.
#[napi]
pub fn preset_n0_disable_relay(builder: &EndpointBuilder) {
    builder.apply_n0_disable_relay();
}

/// Which side of a connection we are.
#[derive(Debug, Clone, Copy)]
#[napi(string_enum)]
pub enum Side {
    Client,
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
#[derive(Debug, Clone)]
#[napi(object)]
pub struct IncomingAddr {
    /// One of "ip" | "relay" | "custom".
    pub kind: String,
    /// `ip:port` for ip, relay URL for relay.
    pub addr: Option<String>,
    /// Remote endpoint id (relay only).
    pub endpoint_id: Option<String>,
    /// Debug description (custom only).
    pub description: Option<String>,
}

impl From<iroh::endpoint::IncomingAddr> for IncomingAddr {
    fn from(addr: iroh::endpoint::IncomingAddr) -> Self {
        match addr {
            iroh::endpoint::IncomingAddr::Ip(socket) => IncomingAddr {
                kind: "ip".into(),
                addr: Some(socket.to_string()),
                endpoint_id: None,
                description: None,
            },
            iroh::endpoint::IncomingAddr::Relay { url, endpoint_id } => IncomingAddr {
                kind: "relay".into(),
                addr: Some(url.to_string()),
                endpoint_id: Some(endpoint_id.to_string()),
                description: None,
            },
            iroh::endpoint::IncomingAddr::Custom(c) => IncomingAddr {
                kind: "custom".into(),
                addr: None,
                endpoint_id: None,
                description: Some(format!("{c:?}")),
            },
            _ => IncomingAddr {
                kind: "custom".into(),
                addr: None,
                endpoint_id: None,
                description: Some("unknown".into()),
            },
        }
    }
}

/// The local address that received an incoming connection.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct IncomingLocalAddr {
    /// One of "ip" | "relay" | "custom".
    pub kind: String,
    pub addr: Option<String>,
    pub description: Option<String>,
}

impl From<iroh::endpoint::LocalTransportAddr> for IncomingLocalAddr {
    fn from(value: iroh::endpoint::LocalTransportAddr) -> Self {
        match value {
            iroh::endpoint::LocalTransportAddr::Ip(ip) => IncomingLocalAddr {
                kind: "ip".into(),
                addr: ip.map(|i| i.to_string()),
                description: None,
            },
            iroh::endpoint::LocalTransportAddr::Relay(url) => IncomingLocalAddr {
                kind: "relay".into(),
                addr: Some(url.to_string()),
                description: None,
            },
            iroh::endpoint::LocalTransportAddr::Custom(c) => IncomingLocalAddr {
                kind: "custom".into(),
                addr: None,
                description: c.map(|c| format!("{c:?}")),
            },
            _ => IncomingLocalAddr {
                kind: "custom".into(),
                addr: None,
                description: None,
            },
        }
    }
}

/// A snapshot value for a single endpoint metric.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct CounterStats {
    pub value: u32,
    pub description: String,
}

/// Flat snapshot of headline connection statistics.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ConnectionStats {
    pub udp_tx_datagrams: i64,
    pub udp_tx_bytes: i64,
    pub udp_rx_datagrams: i64,
    pub udp_rx_bytes: i64,
    pub lost_packets: i64,
    pub lost_bytes: i64,
}

/// Options passed to [`Endpoint::bind`].
///
/// `bind` applies the n0 preset by default. For a custom preset use
/// [`Endpoint::builder`] + the `EndpointBuilder` surface.
#[derive(Debug, Default)]
#[napi(object)]
pub struct EndpointOptions {
    pub bind_addr: Option<String>,
    pub secret_key: Option<Vec<u8>>,
    pub alpns: Option<Vec<Vec<u8>>>,
}

/// An iroh endpoint.
#[napi]
pub struct Endpoint {
    inner: endpoint::Endpoint,
}

impl Endpoint {
    pub(crate) fn raw(&self) -> &endpoint::Endpoint {
        &self.inner
    }
}

#[napi]
impl Endpoint {
    /// Create an endpoint builder (starts empty — apply a preset).
    #[napi]
    pub fn builder() -> EndpointBuilder {
        EndpointBuilder::new(iroh::endpoint::Builder::empty())
    }

    /// Bind a new endpoint. Applies the n0 preset, then the given options.
    /// For a custom preset use [`Endpoint::builder`].
    #[napi(factory)]
    pub async fn bind(
        options: Option<EndpointOptions>,
        relay_mode: Option<&RelayMode>,
    ) -> Result<Self> {
        let options = options.unwrap_or_default();
        let wrapper = EndpointBuilder::new(iroh::endpoint::Builder::empty());
        wrapper.apply_n0();

        if let Some(secret_key) = options.secret_key {
            wrapper.secret_key(secret_key)?;
        }
        if let Some(alpns) = options.alpns {
            wrapper.alpns(alpns);
        }
        if let Some(relay_mode) = relay_mode {
            wrapper.relay_mode(relay_mode);
        }
        if let Some(addr) = options.bind_addr {
            wrapper.bind_addr(addr)?;
        }

        wrapper.bind().await
    }

    /// This endpoint's id.
    #[napi]
    pub fn id(&self) -> crate::EndpointId {
        self.inner.id().into()
    }

    /// The [`EndpointAddr`] for this endpoint.
    #[napi]
    pub fn addr(&self) -> EndpointAddr {
        self.inner.addr().into()
    }

    /// Look up cached information about a remote endpoint, if any.
    #[napi]
    pub async fn remote_addr(&self, id: &crate::EndpointId) -> Result<Option<EndpointAddr>> {
        let id: iroh::EndpointId = id.into();
        let info = match self.inner.remote_info(id).await {
            Some(i) => i,
            None => return Ok(None),
        };
        let id = info.id();
        let addrs = info.into_addrs().map(|a| a.into_addr());
        Ok(Some(iroh::EndpointAddr::from_parts(id, addrs).into()))
    }

    /// Current statistics for this endpoint.
    #[napi]
    pub fn stats(&self) -> std::collections::HashMap<String, CounterStats> {
        use iroh_metrics::{MetricValue, MetricsGroupSet};
        self.inner
            .metrics()
            .iter()
            .map(|(group, item)| {
                let name = format!("{}:{}", group, item.name());
                let value = match item.value() {
                    MetricValue::Counter(v) => u32::try_from(v).unwrap_or(u32::MAX),
                    MetricValue::Gauge(v) => u32::try_from(v.max(0)).unwrap_or(u32::MAX),
                    _ => 0,
                };
                (
                    name,
                    CounterStats {
                        value,
                        description: item.help().to_string(),
                    },
                )
            })
            .collect()
    }

    /// The secret key backing this endpoint's identity.
    #[napi]
    pub fn secret_key(&self) -> SecretKey {
        self.inner.secret_key().clone().into()
    }

    /// Replace the set of advertised ALPNs.
    #[napi]
    pub fn set_alpns(&self, alpns: Vec<Vec<u8>>) {
        self.inner.set_alpns(alpns);
    }

    /// Add an external (manually-known) socket address.
    #[napi]
    pub async fn add_external_addr(&self, addr: String) -> Result<()> {
        let socket = std::net::SocketAddr::from_str(&addr).map_err(anyhow::Error::from)?;
        self.inner.add_external_addr(socket).await;
        Ok(())
    }

    /// Remove a previously-added external address.
    #[napi]
    pub async fn remove_external_addr(&self, addr: String) -> Result<bool> {
        let socket = std::net::SocketAddr::from_str(&addr).map_err(anyhow::Error::from)?;
        Ok(self.inner.remove_external_addr(&socket).await)
    }

    /// The local socket addresses this endpoint is bound to.
    #[napi]
    pub fn bound_sockets(&self) -> Vec<String> {
        self.inner
            .bound_sockets()
            .into_iter()
            .map(|a| a.to_string())
            .collect()
    }

    /// Resolves once the endpoint has a usable home relay.
    #[napi]
    pub async fn online(&self) {
        self.inner.online().await;
    }

    /// Insert (or replace) a relay configuration at runtime.
    #[napi]
    pub async fn insert_relay(&self, config: crate::RelayConfig) -> Result<()> {
        let config: iroh::RelayConfig = config.try_into()?;
        let url = config.url.clone();
        self.inner.insert_relay(url, Arc::new(config)).await;
        Ok(())
    }

    /// Remove a relay configuration at runtime.
    #[napi]
    pub async fn remove_relay(&self, url: String) -> Result<bool> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.inner.remove_relay(&url).await.is_some())
    }

    /// Connect to a remote endpoint via the given ALPN.
    #[napi]
    pub async fn connect(&self, addr: &EndpointAddr, alpn: Vec<u8>) -> Result<Connection> {
        let addr: iroh::EndpointAddr = addr.try_into()?;
        let conn = self
            .inner
            .connect(addr, &alpn)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Connection(Arc::new(conn)))
    }

    /// Begin a connection attempt, returning the in-progress handle.
    #[napi]
    pub async fn connect_pending(&self, addr: &EndpointAddr, alpn: Vec<u8>) -> Result<Connecting> {
        let addr: iroh::EndpointAddr = addr.try_into()?;
        let connecting = self
            .inner
            .connect_with_opts(addr, &alpn, iroh::endpoint::ConnectOptions::default())
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Connecting(Mutex::new(Some(connecting))))
    }

    /// Pull the next incoming connection attempt.
    #[napi]
    pub async fn accept_next(&self) -> Option<Incoming> {
        let inner = self.inner.accept().await?;
        Some(Incoming(Mutex::new(Some(inner))))
    }

    /// Watch for changes to this endpoint's address.
    #[napi(ts_args_type = "callback: (addr: EndpointAddr) => void")]
    pub fn watch_addr(&self, callback: ThreadsafeFunction<EndpointAddr>) -> WatchHandle {
        watch::spawn_watch_addr(self.inner.clone(), callback)
    }

    /// Watch for changes to the connected relays.
    #[napi(ts_args_type = "callback: (relayUrls: Array<string>) => void")]
    pub fn watch_home_relay(&self, callback: ThreadsafeFunction<Vec<String>>) -> WatchHandle {
        watch::spawn_home_relay_watch(self.inner.clone(), callback)
    }

    /// Watch for network-stack changes.
    #[napi(ts_args_type = "callback: () => void")]
    pub fn watch_network_change(&self, callback: ThreadsafeFunction<()>) -> WatchHandle {
        watch::spawn_network_change_watch(self.inner.clone(), callback)
    }

    /// Close the endpoint.
    #[napi]
    pub async fn close(&self) {
        self.inner.close().await;
    }

    /// True if the endpoint is closed.
    #[napi]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

/// An incoming connection that has not yet begun its server-side handshake.
#[napi]
pub struct Incoming(Mutex<Option<iroh::endpoint::Incoming>>);

#[napi]
impl Incoming {
    #[napi]
    pub async fn accept(&self) -> Result<Accepting> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        let accepting = inner.accept().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Accepting(Mutex::new(Some(accepting))))
    }

    #[napi]
    pub async fn refuse(&self) -> Result<()> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        inner.refuse();
        Ok(())
    }

    #[napi]
    pub async fn retry(&self) -> Result<()> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        inner
            .retry()
            .map_err(|e| anyhow::anyhow!("retry failed: {e:?}").into())
    }

    #[napi]
    pub async fn ignore(&self) -> Result<()> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        inner.ignore();
        Ok(())
    }

    #[napi]
    pub async fn local_addr(&self) -> Result<IncomingLocalAddr> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        Ok(inner.local_addr().into())
    }

    #[napi]
    pub async fn remote_addr(&self) -> Result<IncomingAddr> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        Ok(inner.remote_addr().into())
    }

    #[napi]
    pub async fn remote_addr_validated(&self) -> Result<bool> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Incoming already consumed"))?;
        Ok(inner.remote_addr_validated())
    }
}

/// A server-side handshake in progress.
#[napi]
pub struct Accepting(Mutex<Option<iroh::endpoint::Accepting>>);

#[napi]
impl Accepting {
    #[napi]
    pub async fn connect(&self) -> Result<Connection> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Accepting already consumed"))?;
        let conn = inner.await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Connection(Arc::new(conn)))
    }

    #[napi]
    pub async fn alpn(&self) -> Result<Vec<u8>> {
        let mut guard = self.0.lock().await;
        let inner = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Accepting already consumed"))?;
        inner
            .alpn()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }
}

/// A client-side handshake in progress.
#[napi]
pub struct Connecting(Mutex<Option<iroh::endpoint::Connecting>>);

#[napi]
impl Connecting {
    #[napi]
    pub async fn connect(&self) -> Result<Connection> {
        let inner = self
            .0
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow::anyhow!("Connecting already consumed"))?;
        let conn = inner.await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Connection(Arc::new(conn)))
    }

    #[napi]
    pub async fn alpn(&self) -> Result<Vec<u8>> {
        let mut guard = self.0.lock().await;
        let inner = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Connecting already consumed"))?;
        inner
            .alpn()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    #[napi]
    pub async fn remote_id(&self) -> Result<crate::EndpointId> {
        let guard = self.0.lock().await;
        let inner = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Connecting already consumed"))?;
        Ok(inner.remote_id().into())
    }
}

/// An active QUIC connection.
#[napi]
pub struct Connection(Arc<endpoint::Connection>);

#[napi]
impl Connection {
    #[napi]
    pub fn alpn(&self) -> Vec<u8> {
        self.0.alpn().to_vec()
    }

    #[napi]
    pub fn remote_id(&self) -> crate::EndpointId {
        self.0.remote_id().into()
    }

    #[napi]
    pub fn side(&self) -> Side {
        self.0.side().into()
    }

    #[napi]
    pub async fn open_uni(&self) -> Result<SendStream> {
        let s = self
            .0
            .open_uni()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(SendStream::new(s))
    }

    #[napi]
    pub async fn accept_uni(&self) -> Result<RecvStream> {
        let r = self
            .0
            .accept_uni()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(RecvStream::new(r))
    }

    #[napi]
    pub async fn open_bi(&self) -> Result<BiStream> {
        let (s, r) = self
            .0
            .open_bi()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    #[napi]
    pub async fn accept_bi(&self) -> Result<BiStream> {
        let (s, r) = self
            .0
            .accept_bi()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    #[napi]
    pub async fn read_datagram(&self) -> Result<Vec<u8>> {
        let res = self
            .0
            .read_datagram()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(res.to_vec())
    }

    #[napi]
    pub async fn closed(&self) -> String {
        self.0.closed().await.to_string()
    }

    #[napi]
    pub fn close_reason(&self) -> Option<String> {
        self.0.close_reason().map(|s| s.to_string())
    }

    #[napi]
    pub fn close(&self, error_code: BigInt, reason: Vec<u8>) -> Result<()> {
        let code =
            endpoint::VarInt::from_u64(error_code.get_u64().1).map_err(anyhow::Error::from)?;
        self.0.close(code, &reason);
        Ok(())
    }

    #[napi]
    pub fn send_datagram(&self, data: Vec<u8>) -> Result<()> {
        self.0
            .send_datagram(data.into())
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub async fn send_datagram_wait(&self, data: Vec<u8>) -> Result<()> {
        self.0
            .send_datagram_wait(data.into())
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub fn max_datagram_size(&self) -> Option<i64> {
        self.0.max_datagram_size().map(|s| s as i64)
    }

    #[napi]
    pub fn datagram_send_buffer_space(&self) -> i64 {
        self.0.datagram_send_buffer_space() as i64
    }

    #[napi]
    pub fn stable_id(&self) -> u32 {
        self.0.stable_id() as _
    }

    #[napi]
    pub fn rtt(&self) -> Option<i64> {
        self.0
            .paths()
            .iter()
            .find(|p| p.is_selected())
            .map(|p| p.rtt().as_millis() as i64)
    }

    #[napi]
    pub fn stats(&self) -> ConnectionStats {
        let s = self.0.stats();
        ConnectionStats {
            udp_tx_datagrams: s.udp_tx.datagrams as i64,
            udp_tx_bytes: s.udp_tx.bytes as i64,
            udp_rx_datagrams: s.udp_rx.datagrams as i64,
            udp_rx_bytes: s.udp_rx.bytes as i64,
            lost_packets: s.lost_packets as i64,
            lost_bytes: s.lost_bytes as i64,
        }
    }

    #[napi]
    pub fn paths(&self) -> Vec<path::PathSnapshot> {
        path::snapshot_paths(&self.0)
    }

    #[napi(ts_args_type = "callback: (paths: Array<PathSnapshot>) => void")]
    pub fn watch_paths(
        &self,
        callback: ThreadsafeFunction<Vec<path::PathSnapshot>>,
    ) -> WatchHandle {
        watch::spawn_paths_watch((*self.0).clone(), callback)
    }

    #[napi(ts_args_type = "callback: (event: PathEvent) => void")]
    pub fn watch_path_events(&self, callback: ThreadsafeFunction<path::PathEvent>) -> WatchHandle {
        watch::spawn_path_events_watch((*self.0).clone(), callback)
    }

    #[napi]
    pub fn set_max_concurrent_uni_streams(&self, count: BigInt) -> Result<()> {
        let n = endpoint::VarInt::from_u64(count.get_u64().1).map_err(anyhow::Error::from)?;
        self.0.set_max_concurrent_uni_streams(n);
        Ok(())
    }

    #[napi]
    pub fn set_receive_window(&self, count: BigInt) -> Result<()> {
        let n = endpoint::VarInt::from_u64(count.get_u64().1).map_err(anyhow::Error::from)?;
        self.0.set_receive_window(n);
        Ok(())
    }

    #[napi]
    pub fn set_max_concurrent_bi_streams(&self, count: BigInt) -> Result<()> {
        let n = endpoint::VarInt::from_u64(count.get_u64().1).map_err(anyhow::Error::from)?;
        self.0.set_max_concurrent_bi_streams(n);
        Ok(())
    }
}

/// A bidirectional QUIC stream pair.
#[napi]
pub struct BiStream {
    send: SendStream,
    recv: RecvStream,
}

#[napi]
impl BiStream {
    #[napi(getter)]
    pub fn send(&self) -> SendStream {
        self.send.clone()
    }

    #[napi(getter)]
    pub fn recv(&self) -> RecvStream {
        self.recv.clone()
    }
}

/// The outgoing half of a QUIC stream.
#[derive(Clone)]
#[napi]
pub struct SendStream(Arc<Mutex<endpoint::SendStream>>);

impl SendStream {
    fn new(s: endpoint::SendStream) -> Self {
        SendStream(Arc::new(Mutex::new(s)))
    }
}

#[napi]
impl SendStream {
    #[napi]
    pub async fn write(&self, buf: Vec<u8>) -> Result<i64> {
        let mut s = self.0.lock().await;
        let n = s.write(&buf).await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(n as i64)
    }

    #[napi]
    pub async fn write_all(&self, buf: Vec<u8>) -> Result<()> {
        let mut s = self.0.lock().await;
        s.write_all(&buf)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub async fn finish(&self) -> Result<()> {
        let mut s = self.0.lock().await;
        s.finish().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub async fn reset(&self, error_code: BigInt) -> Result<()> {
        let code =
            endpoint::VarInt::from_u64(error_code.get_u64().1).map_err(anyhow::Error::from)?;
        let mut s = self.0.lock().await;
        s.reset(code).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub async fn set_priority(&self, p: i32) -> Result<()> {
        let s = self.0.lock().await;
        s.set_priority(p).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub async fn priority(&self) -> Result<i32> {
        let s = self.0.lock().await;
        s.priority().map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    #[napi]
    pub async fn stopped(&self) -> Result<Option<i64>> {
        let s = self.0.lock().await;
        let res = s.stopped().await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(res.map(|r| r.into_inner() as i64))
    }

    #[napi]
    pub async fn id(&self) -> String {
        self.0.lock().await.id().to_string()
    }
}

/// The incoming half of a QUIC stream.
#[derive(Clone)]
#[napi]
pub struct RecvStream(Arc<Mutex<endpoint::RecvStream>>);

impl RecvStream {
    fn new(s: endpoint::RecvStream) -> Self {
        RecvStream(Arc::new(Mutex::new(s)))
    }
}

#[napi]
impl RecvStream {
    #[napi]
    pub async fn read(&self, size_limit: u32) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; size_limit as usize];
        let mut r = self.0.lock().await;
        let res = r
            .read(&mut buf)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let len = res.unwrap_or(0);
        buf.truncate(len);
        Ok(buf)
    }

    #[napi]
    pub async fn read_exact(&self, size: u32) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; size as usize];
        let mut r = self.0.lock().await;
        r.read_exact(&mut buf)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(buf)
    }

    #[napi]
    pub async fn read_to_end(&self, size_limit: u32) -> Result<Vec<u8>> {
        let mut r = self.0.lock().await;
        let res = r
            .read_to_end(size_limit as usize)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(res)
    }

    #[napi]
    pub async fn id(&self) -> String {
        self.0.lock().await.id().to_string()
    }

    #[napi]
    pub async fn bytes_read(&self) -> Result<i64> {
        let r = self.0.lock().await;
        Ok(r.bytes_read().map_err(|e| anyhow::anyhow!("{e:?}"))? as i64)
    }

    #[napi]
    pub async fn stop(&self, error_code: BigInt) -> Result<()> {
        let code =
            endpoint::VarInt::from_u64(error_code.get_u64().1).map_err(anyhow::Error::from)?;
        let mut r = self.0.lock().await;
        r.stop(code).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(())
    }

    #[napi]
    pub async fn received_reset(&self) -> Result<Option<i64>> {
        let mut r = self.0.lock().await;
        let code = r
            .received_reset()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(code.map(|c| c.into_inner() as i64))
    }
}
