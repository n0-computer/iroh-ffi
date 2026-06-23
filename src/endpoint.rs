use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::Arc};

use iroh::{
    endpoint::{self, presets, presets::Preset as _},
    protocol::AcceptError,
};
use tokio::sync::Mutex;

use crate::{
    AddrChangeCallback, CallbackError, Connecting, EndpointAddr, EndpointId, HomeRelayCallback,
    Incoming, IrohError, NetworkChangeCallback, PathChangeCallback, PathEventCallback,
    PathSnapshot, RelayConfig, RelayMode, SecretKey, Side, WatchHandle, path, watch,
};

/// A mutable handle to an endpoint builder, handed to [`Preset::apply`].
///
/// Mirrors the chainable surface of `iroh::endpoint::Builder` that a preset
/// cares about. The three `apply_*` methods replay the corresponding upstream
/// `iroh::endpoint::presets` impl (which, importantly, install the crypto
/// provider) — a custom preset will almost always call one of them as a
/// baseline before layering its own configuration.
#[derive(uniffi::Object)]
pub struct EndpointBuilder {
    inner: std::sync::Mutex<Option<iroh::endpoint::Builder>>,
}

impl EndpointBuilder {
    pub(crate) fn new(builder: iroh::endpoint::Builder) -> Self {
        Self {
            inner: std::sync::Mutex::new(Some(builder)),
        }
    }

    fn map<F>(&self, f: F)
    where
        F: FnOnce(iroh::endpoint::Builder) -> iroh::endpoint::Builder,
    {
        let mut guard = self.inner.lock().unwrap();
        let builder = guard.take().expect("EndpointBuilder consumed");
        *guard = Some(f(builder));
    }

    pub(crate) fn take_inner(&self) -> Result<iroh::endpoint::Builder, IrohError> {
        self.inner
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| anyhow::anyhow!("EndpointBuilder already consumed").into())
    }
}

#[uniffi::export]
impl EndpointBuilder {
    /// Replay the n0 production preset (relays + discovery + crypto provider).
    pub fn apply_n0(&self) {
        self.map(|b| presets::N0.apply(b));
    }

    /// Replay the minimal preset (crypto provider only, no external deps).
    pub fn apply_minimal(&self) {
        self.map(|b| presets::Minimal.apply(b));
    }

    /// Replay the n0 preset with relays disabled.
    pub fn apply_n0_disable_relay(&self) {
        self.map(|b| presets::N0DisableRelay.apply(b));
    }

    /// Set the endpoint secret key (32 bytes).
    pub fn secret_key(&self, bytes: Vec<u8>) -> Result<(), IrohError> {
        let key: [u8; 32] = AsRef::<[u8]>::as_ref(&bytes)
            .try_into()
            .map_err(|e| anyhow::anyhow!("invalid secret key length: {e:?}"))?;
        let key = iroh::SecretKey::from_bytes(&key);
        self.map(|b| b.secret_key(key));
        Ok(())
    }

    /// Set the advertised ALPNs.
    pub fn alpns(&self, alpns: Vec<Vec<u8>>) {
        self.map(|b| b.alpns(alpns));
    }

    /// Set the relay mode.
    pub fn relay_mode(&self, mode: &RelayMode) {
        let mode = mode.0.clone();
        self.map(|b| b.relay_mode(mode));
    }

    /// Set the address the endpoint binds to (`host:port`).
    pub fn bind_addr(&self, addr: String) -> Result<(), IrohError> {
        let socket = std::net::SocketAddr::from_str(&addr).map_err(anyhow::Error::from)?;
        let builder = self.take_inner()?;
        let builder = builder.bind_addr(socket).map_err(anyhow::Error::from)?;
        *self.inner.lock().unwrap() = Some(builder);
        Ok(())
    }
}

/// Configures a freshly created [`EndpointBuilder`].
///
/// This mirrors the upstream `iroh::endpoint::presets::Preset` trait and is
/// implementable from the foreign language: implement `apply` to configure the
/// builder however you like (typically calling one of the
/// [`EndpointBuilder::apply_n0`] / `apply_minimal` / `apply_n0_disable_relay`
/// baselines first, since those install the crypto provider). The built-in
/// presets are available as [`preset_n0`], [`preset_minimal`], and
/// [`preset_n0_disable_relay`].
#[uniffi::export(with_foreign)]
pub trait Preset: Send + Sync + 'static {
    fn apply(&self, builder: Arc<EndpointBuilder>);
}

struct N0Preset;
impl Preset for N0Preset {
    fn apply(&self, builder: Arc<EndpointBuilder>) {
        builder.apply_n0();
    }
}

struct MinimalPreset;
impl Preset for MinimalPreset {
    fn apply(&self, builder: Arc<EndpointBuilder>) {
        builder.apply_minimal();
    }
}

struct N0DisableRelayPreset;
impl Preset for N0DisableRelayPreset {
    fn apply(&self, builder: Arc<EndpointBuilder>) {
        builder.apply_n0_disable_relay();
    }
}

/// The n0 production preset (relays + discovery).
#[uniffi::export]
pub fn preset_n0() -> Arc<dyn Preset> {
    Arc::new(N0Preset)
}

/// The minimal preset (no external dependencies; good for tests / offline).
#[uniffi::export]
pub fn preset_minimal() -> Arc<dyn Preset> {
    Arc::new(MinimalPreset)
}

/// The n0 preset with relays disabled.
#[uniffi::export]
pub fn preset_n0_disable_relay() -> Arc<dyn Preset> {
    Arc::new(N0DisableRelayPreset)
}

/// Options passed to [`Endpoint::bind`].
#[derive(derive_more::Debug, Default, uniffi::Record)]
pub struct EndpointOptions {
    /// Preset that configures the endpoint builder. Defaults to [`preset_n0`].
    /// Implement the [`Preset`] trait in your language for full control.
    #[debug(skip)]
    #[uniffi(default = None)]
    pub preset: Option<Arc<dyn Preset>>,
    /// Override the address the endpoint binds to. Accepts any standard
    /// `host:port` form (IPv4 or IPv6).
    #[uniffi(default = None)]
    pub bind_addr: Option<String>,
    /// Provide a specific secret key, identifying this endpoint. Must be 32 bytes long.
    #[uniffi(default = None)]
    pub secret_key: Option<Vec<u8>>,
    /// ALPN protocols advertised on the underlying TLS handshake. Independent of
    /// the per-protocol handlers in `protocols`; useful for client-only setups
    /// or for declaring extra ALPNs.
    #[uniffi(default = None)]
    pub alpns: Option<Vec<Vec<u8>>>,
    /// Override which relays the endpoint uses. Defaults to whatever the
    /// chosen [`Preset`] configures.
    #[uniffi(default = None)]
    pub relay_mode: Option<Arc<RelayMode>>,
    /// Custom protocols to accept on this endpoint, keyed by ALPN. If provided,
    /// an internal router is spawned to dispatch incoming connections to the
    /// supplied handlers.
    #[uniffi(default = None)]
    pub protocols: Option<HashMap<String, Arc<dyn ProtocolCreator>>>,
}

#[uniffi::export(with_foreign)]
pub trait ProtocolCreator: std::fmt::Debug + Send + Sync + 'static {
    fn create(&self, endpoint: Arc<Endpoint>) -> Arc<dyn ProtocolHandler>;
}

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
    async fn accept(&self, conn: iroh::endpoint::Connection) -> Result<(), AcceptError> {
        let this = self.clone();
        this.handler
            .accept(Arc::new(conn.into()))
            .await
            .map_err(AcceptError::from_err)?;
        Ok(())
    }

    async fn shutdown(&self) {
        let this = self.clone();
        this.handler.shutdown().await;
    }
}

/// A snapshot value for a single endpoint metric.
#[derive(Debug, uniffi::Record)]
pub struct CounterStats {
    /// The counter / gauge value.
    pub value: u32,
    /// The metric description.
    pub description: String,
}

/// Flat snapshot of the headline numbers from `noq::ConnectionStats`.
///
/// Counters are `i64` (not `u64`) so Kotlin sees `Long`, not `ULong`.
#[derive(Debug, uniffi::Record)]
pub struct ConnectionStats {
    /// Total UDP datagrams transmitted.
    pub udp_tx_datagrams: i64,
    /// Total UDP bytes transmitted.
    pub udp_tx_bytes: i64,
    /// Total UDP datagrams received.
    pub udp_rx_datagrams: i64,
    /// Total UDP bytes received.
    pub udp_rx_bytes: i64,
    /// Total packets considered lost.
    pub lost_packets: i64,
    /// Total bytes considered lost.
    pub lost_bytes: i64,
}

/// An iroh endpoint.
///
/// Bind one with [`Endpoint::bind`]. Provide protocol handlers via
/// [`EndpointOptions::protocols`] to dispatch incoming connections.
#[derive(Clone, uniffi::Object)]
pub struct Endpoint {
    inner: endpoint::Endpoint,
    router: Option<iroh::protocol::Router>,
}

impl Endpoint {
    pub fn new(ep: endpoint::Endpoint) -> Self {
        Endpoint {
            inner: ep,
            router: None,
        }
    }

    pub(crate) fn raw(&self) -> &endpoint::Endpoint {
        &self.inner
    }
}

#[uniffi::export]
impl Endpoint {
    /// Bind a new endpoint with the given options.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn bind(options: EndpointOptions) -> Result<Self, IrohError> {
        // Start from an empty builder; the preset installs the baseline
        // (crucially, the crypto provider). Explicit option fields are then
        // layered on top so they always win.
        let wrapper = Arc::new(EndpointBuilder::new(iroh::endpoint::Builder::empty()));
        let preset = options.preset.unwrap_or_else(preset_n0);
        preset.apply(wrapper.clone());

        if let Some(secret_key) = options.secret_key {
            wrapper.secret_key(secret_key)?;
        }
        if let Some(alpns) = options.alpns {
            wrapper.alpns(alpns);
        }
        if let Some(relay_mode) = options.relay_mode {
            wrapper.relay_mode(&relay_mode);
        }
        if let Some(addr) = options.bind_addr {
            wrapper.bind_addr(addr)?;
        }

        let builder = wrapper.take_inner()?;
        let endpoint = builder.bind().await?;

        let router = match options.protocols {
            Some(protocols) if !protocols.is_empty() => {
                let mut router_builder = iroh::protocol::Router::builder(endpoint.clone());
                let endpoint_wrapper = Arc::new(Endpoint::new(endpoint.clone()));
                for (alpn, creator) in protocols {
                    let handler = creator.create(endpoint_wrapper.clone());
                    router_builder = router_builder.accept(alpn, ProtocolWrapper { handler });
                }
                Some(router_builder.spawn())
            }
            _ => None,
        };

        Ok(Endpoint {
            inner: endpoint,
            router,
        })
    }

    /// The [`EndpointId`] of this endpoint.
    pub fn id(&self) -> Arc<EndpointId> {
        Arc::new(self.inner.id().into())
    }

    /// The [`EndpointAddr`] for this endpoint (id + currently known addresses).
    pub fn addr(&self) -> Arc<EndpointAddr> {
        Arc::new(self.inner.addr().into())
    }

    /// Look up cached information about a remote endpoint, if any.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_addr(&self, id: &EndpointId) -> Option<Arc<EndpointAddr>> {
        let info = self.inner.remote_info(id.into()).await?;
        let id = info.id();
        let addrs = info.into_addrs().map(|a| a.into_addr());
        Some(Arc::new(iroh::EndpointAddr::from_parts(id, addrs).into()))
    }

    /// Get current statistics for this endpoint.
    ///
    /// Keys are `"<group>:<metric>"`. Counter / gauge values are saturating-cast to `u32`.
    pub fn stats(&self) -> HashMap<String, CounterStats> {
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

    /// Connect to a remote endpoint via the given ALPN.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect(&self, addr: &EndpointAddr, alpn: &[u8]) -> Result<Connection, IrohError> {
        let addr: iroh::EndpointAddr = addr.clone().try_into()?;
        let conn = self.inner.connect(addr, alpn).await?;
        Ok(Connection(conn))
    }

    /// Shut down the endpoint (and, if present, the protocol router).
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn close(&self) -> Result<(), IrohError> {
        if let Some(router) = &self.router {
            router.shutdown().await?;
        } else {
            self.inner.close().await;
        }
        Ok(())
    }

    /// Returns true if the endpoint has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// The [`SecretKey`] backing this endpoint's identity.
    pub fn secret_key(&self) -> Arc<SecretKey> {
        Arc::new(self.inner.secret_key().clone().into())
    }

    /// Replace the set of ALPNs advertised by this endpoint.
    pub fn set_alpns(&self, alpns: Vec<Vec<u8>>) {
        self.inner.set_alpns(alpns);
    }

    /// Add an external (manually-known) socket address that this endpoint is
    /// reachable on. Useful when running behind a static NAT / load balancer.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_external_addr(&self, addr: String) -> Result<(), IrohError> {
        let socket = std::net::SocketAddr::from_str(&addr).map_err(anyhow::Error::from)?;
        self.inner.add_external_addr(socket).await;
        Ok(())
    }

    /// Remove a previously-added external address. Returns true if an entry was
    /// removed.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remove_external_addr(&self, addr: String) -> Result<bool, IrohError> {
        let socket = std::net::SocketAddr::from_str(&addr).map_err(anyhow::Error::from)?;
        Ok(self.inner.remove_external_addr(&socket).await)
    }

    /// The local socket addresses this endpoint is bound to.
    pub fn bound_sockets(&self) -> Vec<String> {
        self.inner
            .bound_sockets()
            .into_iter()
            .map(|a| a.to_string())
            .collect()
    }

    /// Resolves once the endpoint has a usable home relay.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn online(&self) {
        self.inner.online().await;
    }

    /// Insert (or replace) a relay configuration at runtime.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn insert_relay(&self, config: RelayConfig) -> Result<(), IrohError> {
        let config: iroh::RelayConfig = config.try_into()?;
        let url = config.url.clone();
        self.inner.insert_relay(url, Arc::new(config)).await;
        Ok(())
    }

    /// Remove a relay configuration at runtime. Returns true if a relay was
    /// removed.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remove_relay(&self, url: String) -> Result<bool, IrohError> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.inner.remove_relay(&url).await.is_some())
    }

    /// Pull the next incoming connection attempt from the accept queue.
    ///
    /// Returns `None` once the endpoint is closed. Use this for a custom accept
    /// loop instead of (or in addition to) registering protocol handlers via
    /// [`EndpointOptions::protocols`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_next(&self) -> Option<Arc<Incoming>> {
        let incoming = self.inner.accept().await?;
        Some(Arc::new(Incoming::new(incoming)))
    }

    /// Begin a connection attempt to `addr` for `alpn`, returning the
    /// in-progress [`Connecting`] state.
    ///
    /// Unlike [`Self::connect`], which awaits the handshake before returning,
    /// this exposes the pre-handshake handle so the caller can inspect ALPN or
    /// drop the attempt explicitly.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect_pending(
        &self,
        addr: &EndpointAddr,
        alpn: &[u8],
    ) -> Result<Connecting, IrohError> {
        let addr: iroh::EndpointAddr = addr.clone().try_into()?;
        let connecting = self
            .inner
            .connect_with_opts(addr, alpn, iroh::endpoint::ConnectOptions::default())
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(Connecting::new(connecting))
    }

    /// Register a callback that fires whenever the endpoint's [`EndpointAddr`]
    /// changes (relay home rotates, IP discovered, etc.). The returned
    /// [`WatchHandle`] cancels the watcher when dropped or when its `stop()`
    /// method is called.
    pub fn watch_addr(&self, callback: Arc<dyn AddrChangeCallback>) -> Arc<WatchHandle> {
        Arc::new(watch::spawn_watch_addr(self.inner.clone(), callback))
    }

    /// Register a callback that fires whenever the list of relays this endpoint
    /// is currently connected to changes.
    pub fn watch_home_relay(&self, callback: Arc<dyn HomeRelayCallback>) -> Arc<WatchHandle> {
        Arc::new(watch::spawn_home_relay_watch(self.inner.clone(), callback))
    }

    /// Register a callback that fires every time the underlying network stack
    /// reports a change (interface up/down, NAT change, roaming, etc.).
    pub fn watch_network_change(
        &self,
        callback: Arc<dyn NetworkChangeCallback>,
    ) -> Arc<WatchHandle> {
        Arc::new(watch::spawn_network_change_watch(
            self.inner.clone(),
            callback,
        ))
    }
}

/// An active QUIC connection to a remote endpoint.
#[derive(uniffi::Object)]
pub struct Connection(endpoint::Connection);

impl From<endpoint::Connection> for Connection {
    fn from(value: endpoint::Connection) -> Self {
        Self(value)
    }
}

#[uniffi::export]
impl Connection {
    /// The ALPN protocol negotiated for this connection.
    pub fn alpn(&self) -> Vec<u8> {
        self.0.alpn().to_vec()
    }

    /// Open a new unidirectional outgoing stream.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn open_uni(&self) -> Result<SendStream, IrohError> {
        let s = self.0.open_uni().await?;
        Ok(SendStream::new(s))
    }

    /// Accept the next incoming unidirectional stream.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_uni(&self) -> Result<RecvStream, IrohError> {
        let r = self.0.accept_uni().await?;
        Ok(RecvStream::new(r))
    }

    /// Open a new bidirectional outgoing stream.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn open_bi(&self) -> Result<BiStream, IrohError> {
        let (s, r) = self.0.open_bi().await?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    /// Accept the next incoming bidirectional stream.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_bi(&self) -> Result<BiStream, IrohError> {
        let (s, r) = self.0.accept_bi().await?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    /// Read the next datagram from the connection.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_datagram(&self) -> Result<Vec<u8>, IrohError> {
        let res = self.0.read_datagram().await?;
        Ok(res.to_vec())
    }

    /// Wait for the connection to be closed, returning the cause.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn closed(&self) -> String {
        let err = self.0.closed().await;
        err.to_string()
    }

    /// If the connection is closed, the reason why. None if still open.
    pub fn close_reason(&self) -> Option<String> {
        self.0.close_reason().map(|s| s.to_string())
    }

    /// Close the connection immediately with the given application error code.
    ///
    /// Signed for Kotlin/Swift ergonomics; negative values are rejected.
    pub fn close(&self, error_code: i64, reason: &[u8]) -> Result<(), IrohError> {
        let unsigned =
            u64::try_from(error_code).map_err(|_| anyhow::anyhow!("error_code must be >= 0"))?;
        let code = endpoint::VarInt::from_u64(unsigned)?;
        self.0.close(code, reason);
        Ok(())
    }

    /// Send a datagram on this connection.
    pub fn send_datagram(&self, data: Vec<u8>) -> Result<(), IrohError> {
        self.0.send_datagram(data.into())?;
        Ok(())
    }

    /// Maximum size of a datagram that can currently be sent.
    pub fn max_datagram_size(&self) -> Option<u64> {
        self.0.max_datagram_size().map(|s| s as _)
    }

    /// Bytes available in the datagram send buffer.
    pub fn datagram_send_buffer_space(&self) -> u64 {
        self.0.datagram_send_buffer_space() as _
    }

    /// The [`EndpointId`] of the remote peer.
    pub fn remote_id(&self) -> Arc<EndpointId> {
        Arc::new(self.0.remote_id().into())
    }

    /// A stable identifier for this connection.
    pub fn stable_id(&self) -> u64 {
        self.0.stable_id() as _
    }

    /// Current best estimate of this connection's RTT on the selected path,
    /// in milliseconds. `None` if no path is currently selected.
    pub fn rtt(&self) -> Option<u64> {
        self.0
            .paths()
            .iter()
            .find(|p| p.is_selected())
            .map(|p| p.rtt().as_millis() as u64)
    }

    /// A flat snapshot of the most useful headline statistics for this connection.
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

    /// Like [`Connection::send_datagram`] but waits for capacity if the send
    /// buffer is full.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn send_datagram_wait(&self, data: Vec<u8>) -> Result<(), IrohError> {
        self.0.send_datagram_wait(data.into()).await?;
        Ok(())
    }

    /// Which side of the connection we are (client or server).
    pub fn side(&self) -> Side {
        self.0.side().into()
    }

    /// A snapshot of all currently open network paths for this connection.
    pub fn paths(&self) -> Vec<PathSnapshot> {
        path::snapshot_paths(&self.0)
    }

    /// Register a callback that fires with the current set of open paths
    /// whenever the path list (or selected path) changes.
    pub fn watch_paths(&self, callback: Arc<dyn PathChangeCallback>) -> Arc<WatchHandle> {
        Arc::new(path::spawn_paths_watch(self.0.clone(), callback))
    }

    /// Register a callback that fires for each individual path event (path
    /// opened, closed, selected, or lagged).
    pub fn watch_path_events(&self, callback: Arc<dyn PathEventCallback>) -> Arc<WatchHandle> {
        Arc::new(path::spawn_path_events_watch(self.0.clone(), callback))
    }

    /// Set the maximum number of concurrent incoming unidirectional streams.
    pub fn set_max_concurrent_uni_streams(&self, count: u64) -> Result<(), IrohError> {
        let n = endpoint::VarInt::from_u64(count)?;
        self.0.set_max_concurrent_uni_streams(n);
        Ok(())
    }

    /// Set the receive window for this connection.
    pub fn set_receive_window(&self, count: u64) -> Result<(), IrohError> {
        let n = endpoint::VarInt::from_u64(count)?;
        self.0.set_receive_window(n);
        Ok(())
    }

    /// Set the maximum number of concurrent incoming bidirectional streams.
    pub fn set_max_concurrent_bi_streams(&self, count: u64) -> Result<(), IrohError> {
        let n = endpoint::VarInt::from_u64(count)?;
        self.0.set_max_concurrent_bi_streams(n);
        Ok(())
    }
}

/// A bidirectional QUIC stream pair.
#[derive(uniffi::Object)]
pub struct BiStream {
    send: SendStream,
    recv: RecvStream,
}

#[uniffi::export]
impl BiStream {
    pub fn send(&self) -> SendStream {
        self.send.clone()
    }

    pub fn recv(&self) -> RecvStream {
        self.recv.clone()
    }
}

/// The outgoing half of a QUIC stream.
#[derive(Clone, uniffi::Object)]
pub struct SendStream(Arc<Mutex<endpoint::SendStream>>);

impl SendStream {
    fn new(s: endpoint::SendStream) -> Self {
        SendStream(Arc::new(Mutex::new(s)))
    }
}

#[uniffi::export]
impl SendStream {
    /// Write some bytes, returning the number actually written.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn write(&self, buf: &[u8]) -> Result<u64, IrohError> {
        let mut s = self.0.lock().await;
        let written = s.write(buf).await?;
        Ok(written as _)
    }

    /// Write all bytes, looping as needed.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn write_all(&self, buf: &[u8]) -> Result<(), IrohError> {
        let mut s = self.0.lock().await;
        s.write_all(buf).await?;
        Ok(())
    }

    /// Signal that no more data will be sent on this stream.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn finish(&self) -> Result<(), IrohError> {
        let mut s = self.0.lock().await;
        s.finish()?;
        Ok(())
    }

    /// Abort the stream with the given error code.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn reset(&self, error_code: u64) -> Result<(), IrohError> {
        let error_code = endpoint::VarInt::from_u64(error_code)?;
        let mut s = self.0.lock().await;
        s.reset(error_code)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn set_priority(&self, p: i32) -> Result<(), IrohError> {
        let s = self.0.lock().await;
        s.set_priority(p)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn priority(&self) -> Result<i32, IrohError> {
        let s = self.0.lock().await;
        let p = s.priority()?;
        Ok(p)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stopped(&self) -> Result<Option<u64>, IrohError> {
        let s = self.0.lock().await;
        let res = s.stopped().await?;
        Ok(res.map(|r| r.into_inner()))
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn id(&self) -> String {
        let r = self.0.lock().await;
        r.id().to_string()
    }
}

/// The incoming half of a QUIC stream.
#[derive(Clone, uniffi::Object)]
pub struct RecvStream(Arc<Mutex<endpoint::RecvStream>>);

impl RecvStream {
    fn new(s: endpoint::RecvStream) -> Self {
        RecvStream(Arc::new(Mutex::new(s)))
    }
}

#[uniffi::export]
impl RecvStream {
    /// Read up to `size_limit` bytes into a fresh buffer.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read(&self, size_limit: u32) -> Result<Vec<u8>, IrohError> {
        let mut buf = vec![0u8; size_limit as _];
        let mut r = self.0.lock().await;
        let res = r.read(&mut buf).await?;
        let len = res.unwrap_or(0);
        buf.truncate(len);
        Ok(buf)
    }

    /// Read exactly `size` bytes, erroring if the stream ends early.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_exact(&self, size: u32) -> Result<Vec<u8>, IrohError> {
        let mut buf = vec![0u8; size as _];
        let mut r = self.0.lock().await;
        r.read_exact(&mut buf).await?;
        Ok(buf)
    }

    /// Read until end-of-stream, with `size_limit` as a maximum.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_to_end(&self, size_limit: u32) -> Result<Vec<u8>, IrohError> {
        let mut r = self.0.lock().await;
        let res = r.read_to_end(size_limit as _).await?;
        Ok(res)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn id(&self) -> String {
        let r = self.0.lock().await;
        r.id().to_string()
    }

    /// Total bytes read from this stream so far.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn bytes_read(&self) -> Result<u64, IrohError> {
        let r = self.0.lock().await;
        Ok(r.bytes_read()?)
    }

    /// Stop the incoming stream with an error code.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stop(&self, error_code: u64) -> Result<(), IrohError> {
        let error_code = endpoint::VarInt::from_u64(error_code)?;
        let mut r = self.0.lock().await;
        r.stop(error_code)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn received_reset(&self) -> Result<Option<u64>, IrohError> {
        let mut r = self.0.lock().await;
        let code = r.received_reset().await?;
        Ok(code.map(|c| c.into_inner()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A user-implemented [`Preset`]: minimal baseline + a custom ALPN.
    #[derive(Debug)]
    struct CustomPreset;
    impl Preset for CustomPreset {
        fn apply(&self, builder: Arc<EndpointBuilder>) {
            builder.apply_minimal();
            builder.alpns(vec![b"custom/preset/1".to_vec()]);
        }
    }

    #[tokio::test]
    async fn test_custom_preset() {
        let ep = Endpoint::bind(EndpointOptions {
            preset: Some(Arc::new(CustomPreset)),
            ..Default::default()
        })
        .await
        .unwrap();
        assert!(!ep.bound_sockets().is_empty());
        ep.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_bind() {
        let ep = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_minimal()),
            ..Default::default()
        })
        .await
        .unwrap();
        let id = ep.id();
        println!("{id}");
        assert!(!ep.bound_sockets().is_empty(), "should have bound sockets");
        let secret = ep.secret_key();
        assert_eq!(secret.public().to_bytes(), id.to_bytes());
        ep.close().await.unwrap();
        assert!(ep.is_closed());
    }

    #[tokio::test]
    async fn test_side_paths_compile() {
        // Surface-level smoke test: the new accept/path types must compile and
        // be callable. End-to-end connection establishment lives in higher-level
        // language-binding tests.
        let ep = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_minimal()),
            ..Default::default()
        })
        .await
        .unwrap();
        // accept_next polled with timeout: just confirm it returns a future of
        // Option<Arc<Incoming>>.
        let timeout = tokio::time::sleep(std::time::Duration::from_millis(10));
        tokio::pin!(timeout);
        tokio::select! {
            _ = &mut timeout => {}
            _next = ep.accept_next() => {}
        }
        ep.close().await.unwrap();
    }

    const TEST_ALPN: &[u8] = b"iroh-ffi/test/0";

    /// Full end-to-end: two endpoints, direct (no relay) connection, bi-stream
    /// echo, datagram round-trip, and connection introspection. This is the
    /// canonical connectivity test mirrored across every binding language.
    #[tokio::test]
    async fn test_connect_echo_roundtrip() {
        let server = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_n0()),
            alpns: Some(vec![TEST_ALPN.to_vec()]),
            relay_mode: Some(Arc::new(RelayMode::disabled())),
            ..Default::default()
        })
        .await
        .unwrap();
        let server_addr = server.addr();
        let server_id = server.id();

        let server_task = {
            let server = server.clone();
            tokio::spawn(async move {
                let incoming = server.accept_next().await.expect("incoming");
                let accepting = incoming.accept().await.unwrap();
                let conn = accepting.connect().await.unwrap();
                assert!(matches!(conn.side(), Side::Server));
                assert_eq!(conn.alpn(), TEST_ALPN);

                let bi = conn.accept_bi().await.unwrap();
                let recv = bi.recv();
                let send = bi.send();
                let msg = recv.read_to_end(64).await.unwrap();
                send.write_all(&msg).await.unwrap();
                send.finish().await.unwrap();

                // datagram echo
                let dg = conn.read_datagram().await.unwrap();
                conn.send_datagram(dg).unwrap();

                conn.closed().await;
            })
        };

        let client = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_n0()),
            relay_mode: Some(Arc::new(RelayMode::disabled())),
            ..Default::default()
        })
        .await
        .unwrap();

        let conn = client.connect(&server_addr, TEST_ALPN).await.unwrap();
        assert!(matches!(conn.side(), Side::Client));
        assert_eq!(conn.remote_id().to_string(), server_id.to_string());
        assert!(!conn.paths().is_empty());

        let bi = conn.open_bi().await.unwrap();
        let send = bi.send();
        let recv = bi.recv();
        send.write_all(b"hello iroh").await.unwrap();
        send.finish().await.unwrap();
        let echoed = recv.read_to_end(64).await.unwrap();
        assert_eq!(echoed, b"hello iroh");

        conn.send_datagram(b"ping".to_vec()).unwrap();
        let pong = conn.read_datagram().await.unwrap();
        assert_eq!(pong, b"ping");

        let stats = conn.stats();
        assert!(stats.udp_tx_datagrams > 0);

        conn.close(0, b"bye").unwrap();
        server_task.await.unwrap();
        client.close().await.unwrap();
        server.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_endpoint_ticket_roundtrip() {
        let ep = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_minimal()),
            ..Default::default()
        })
        .await
        .unwrap();
        let addr = ep.addr();
        let ticket = crate::EndpointTicket::from_addr(&addr).unwrap();
        let s = ticket.to_string();
        let parsed = crate::EndpointTicket::from_string(s.clone()).unwrap();
        assert_eq!(parsed.endpoint_addr().id().to_string(), ep.id().to_string());
        ep.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_uni_stream() {
        let server = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_n0()),
            alpns: Some(vec![TEST_ALPN.to_vec()]),
            relay_mode: Some(Arc::new(RelayMode::disabled())),
            ..Default::default()
        })
        .await
        .unwrap();
        let server_addr = server.addr();

        let server_task = {
            let server = server.clone();
            tokio::spawn(async move {
                let incoming = server.accept_next().await.expect("incoming");
                let conn = incoming.accept().await.unwrap().connect().await.unwrap();
                let recv = conn.accept_uni().await.unwrap();
                let msg = recv.read_to_end(32).await.unwrap();
                assert_eq!(msg, b"unidirectional");
            })
        };

        let client = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_n0()),
            relay_mode: Some(Arc::new(RelayMode::disabled())),
            ..Default::default()
        })
        .await
        .unwrap();
        let conn = client.connect(&server_addr, TEST_ALPN).await.unwrap();
        let send = conn.open_uni().await.unwrap();
        send.write_all(b"unidirectional").await.unwrap();
        send.finish().await.unwrap();

        server_task.await.unwrap();
        client.close().await.unwrap();
        server.close().await.unwrap();
    }
}
