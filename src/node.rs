use std::sync::Mutex;
use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc, time::Duration};

use iroh_blobs::{
    downloader::Downloader, net_protocol::Blobs, provider::EventSender, store::GcConfig,
    util::local_pool::LocalPool,
};
use iroh_docs::protocol::Docs;
use iroh_gossip::net::Gossip;
use iroh_node_util::rpc::server::AbstractNode;
use quic_rpc::{transport::flume::FlumeConnector, RpcClient, RpcServer};
use tokio_util::task::AbortOnDropHandle;

use crate::{
    BlobProvideEventCallback, CallbackError, Connecting, Endpoint, IrohError, NodeAddr, PublicKey,
};

/// Stats counter
#[derive(Debug, uniffi::Record)]
pub struct CounterStats {
    /// The counter value
    pub value: u32,
    /// The counter description
    pub description: String,
}

/// Information about a direct address.
#[derive(Debug, Clone, uniffi::Object)]
pub struct DirectAddrInfo(pub(crate) iroh::endpoint::DirectAddrInfo);

#[uniffi::export]
impl DirectAddrInfo {
    /// Get the reported address
    pub fn addr(&self) -> String {
        self.0.addr.to_string()
    }

    /// Get the reported latency, if it exists
    pub fn latency(&self) -> Option<Duration> {
        self.0.latency
    }

    /// Get the last control message received by this node
    pub fn last_control(&self) -> Option<LatencyAndControlMsg> {
        self.0
            .last_control
            .map(|(latency, control_msg)| LatencyAndControlMsg {
                latency,
                control_msg: control_msg.to_string(),
            })
    }

    /// Get how long ago the last payload message was received for this node
    pub fn last_payload(&self) -> Option<Duration> {
        self.0.last_payload
    }
}

/// The latency and type of the control message
#[derive(Debug, uniffi::Record)]
pub struct LatencyAndControlMsg {
    /// The latency of the control message
    pub latency: Duration,
    /// The type of control message, represented as a string
    pub control_msg: String,
    // control_msg: ControlMsg
}

// TODO: enable and use for `LatencyAndControlMsg.control_msg` field when iroh core makes this public
// The kinds of control messages that can be sent
// pub use iroh::magicsock::ControlMsg;

/// Information about a remote node
#[derive(Debug, uniffi::Record)]
pub struct RemoteInfo {
    /// The node identifier of the endpoint. Also a public key.
    pub node_id: Arc<PublicKey>,
    /// Relay url, if available.
    pub relay_url: Option<String>,
    /// List of addresses at which this node might be reachable, plus any latency information we
    /// have about that address and the last time the address was used.
    pub addrs: Vec<Arc<DirectAddrInfo>>,
    /// The type of connection we have to the peer, either direct or over relay.
    pub conn_type: Arc<ConnectionType>,
    /// The latency of the `conn_type`.
    pub latency: Option<Duration>,
    /// Duration since the last time this peer was used.
    pub last_used: Option<Duration>,
}

impl From<iroh::endpoint::RemoteInfo> for RemoteInfo {
    fn from(value: iroh::endpoint::RemoteInfo) -> Self {
        RemoteInfo {
            node_id: Arc::new(value.node_id.into()),
            relay_url: value.relay_url.map(|info| info.relay_url.to_string()),
            addrs: value
                .addrs
                .iter()
                .map(|a| Arc::new(DirectAddrInfo(a.clone())))
                .collect(),
            conn_type: Arc::new(value.conn_type.into()),
            latency: value.latency,
            last_used: value.last_used,
        }
    }
}

/// The type of the connection
#[derive(Debug, uniffi::Enum)]
pub enum ConnType {
    /// Indicates you have a UDP connection.
    Direct,
    /// Indicates you have a relayed connection.
    Relay,
    /// Indicates you have an unverified UDP connection, and a relay connection for backup.
    Mixed,
    /// Indicates you have no proof of connection.
    None,
}

/// The type of connection we have to the node
#[derive(Debug, uniffi::Object)]
pub enum ConnectionType {
    /// Direct UDP connection
    Direct(String),
    /// Relay connection
    Relay(String),
    /// Both a UDP and a Relay connection are used.
    ///
    /// This is the case if we do have a UDP address, but are missing a recent confirmation that
    /// the address works.
    Mixed(String, String),
    /// We have no verified connection to this PublicKey
    None,
}

#[uniffi::export]
impl ConnectionType {
    /// Whether connection is direct, relay, mixed, or none
    pub fn r#type(&self) -> ConnType {
        match self {
            ConnectionType::Direct(_) => ConnType::Direct,
            ConnectionType::Relay(_) => ConnType::Relay,
            ConnectionType::Mixed(..) => ConnType::Mixed,
            ConnectionType::None => ConnType::None,
        }
    }

    /// Return the socket address if this is a direct connection
    pub fn as_direct(&self) -> String {
        match self {
            ConnectionType::Direct(addr) => addr.clone(),
            _ => panic!("ConnectionType type is not 'Direct'"),
        }
    }

    /// Return the derp url if this is a relay connection
    pub fn as_relay(&self) -> String {
        match self {
            ConnectionType::Relay(url) => url.clone(),
            _ => panic!("ConnectionType is not `Relay`"),
        }
    }

    /// Return the socket address and DERP url if this is a mixed connection
    pub fn as_mixed(&self) -> ConnectionTypeMixed {
        match self {
            ConnectionType::Mixed(addr, url) => ConnectionTypeMixed {
                addr: addr.clone(),
                relay_url: url.clone(),
            },
            _ => panic!("ConnectionType is not `Relay`"),
        }
    }
}

/// The socket address and url of the mixed connection
#[derive(Debug, uniffi::Record)]
pub struct ConnectionTypeMixed {
    /// Address of the node
    pub addr: String,
    /// Url of the relay node to which the node is connected
    pub relay_url: String,
}

impl From<iroh::endpoint::ConnectionType> for ConnectionType {
    fn from(value: iroh::endpoint::ConnectionType) -> Self {
        match value {
            iroh::endpoint::ConnectionType::Direct(addr) => {
                ConnectionType::Direct(addr.to_string())
            }
            iroh::endpoint::ConnectionType::Mixed(addr, url) => {
                ConnectionType::Mixed(addr.to_string(), url.to_string())
            }
            iroh::endpoint::ConnectionType::Relay(url) => ConnectionType::Relay(url.to_string()),
            iroh::endpoint::ConnectionType::None => ConnectionType::None,
        }
    }
}
/// Options passed to [`IrohNode.new`]. Controls the behaviour of an iroh node.
#[derive(derive_more::Debug, uniffi::Record)]
pub struct NodeOptions {
    /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
    /// Set to 0 to disable gc
    #[uniffi(default = None)]
    pub gc_interval_millis: Option<u64>,
    /// Provide a callback to hook into events when the blobs component adds and provides blobs.
    #[debug("BlobProvideEventCallback")]
    #[uniffi(default = None)]
    pub blob_events: Option<Arc<dyn BlobProvideEventCallback>>,
    /// Should docs be enabled? Defaults to `false`.
    #[uniffi(default = false)]
    pub enable_docs: bool,
    /// Overwrites the default IPv4 address to bind to
    #[uniffi(default = None)]
    pub ipv4_addr: Option<String>,
    /// Overwrites the default IPv6 address to bind to
    #[uniffi(default = None)]
    pub ipv6_addr: Option<String>,
    /// Configure the node discovery. Defaults to the default set of config
    #[uniffi(default = None)]
    pub node_discovery: Option<NodeDiscoveryConfig>,
    /// Provide a specific secret key, identifying this node. Must be 32 bytes long.
    #[uniffi(default = None)]
    pub secret_key: Option<Vec<u8>>,

    #[uniffi(default = None)]
    pub protocols: Option<HashMap<Vec<u8>, Arc<dyn ProtocolCreator>>>,
}

#[uniffi::export(with_foreign)]
pub trait ProtocolCreator: std::fmt::Debug + Send + Sync + 'static {
    fn create(&self, endpoint: Arc<Endpoint>) -> Arc<dyn ProtocolHandler>;
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait ProtocolHandler: Send + Sync + 'static {
    async fn accept(&self, conn: Arc<Connecting>) -> Result<(), CallbackError>;
    async fn shutdown(&self);
}

#[derive(derive_more::Debug, Clone)]
struct ProtocolWrapper {
    #[debug("handler")]
    handler: Arc<dyn ProtocolHandler>,
}

impl iroh::protocol::ProtocolHandler for ProtocolWrapper {
    fn accept(
        &self,
        conn: iroh::endpoint::Connecting,
    ) -> futures_lite::future::Boxed<anyhow::Result<()>> {
        let this = self.clone();
        Box::pin(async move {
            let conn = Connecting::new(conn);
            this.handler.accept(Arc::new(conn)).await?;
            Ok(())
        })
    }

    fn shutdown(&self) -> futures_lite::future::Boxed<()> {
        let this = self.clone();
        Box::pin(async move {
            this.handler.shutdown().await;
        })
    }
}

impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            gc_interval_millis: Some(0),
            blob_events: None,
            enable_docs: false,
            ipv4_addr: None,
            ipv6_addr: None,
            node_discovery: None,
            secret_key: None,
            protocols: None,
        }
    }
}

#[derive(Debug, Default, uniffi::Enum)]
pub enum NodeDiscoveryConfig {
    /// Use no node discovery mechanism.
    None,
    /// Use the default discovery mechanism.
    ///
    /// This uses two discovery services concurrently:
    ///
    /// - It publishes to a pkarr service operated by [number 0] which makes the information
    ///   available via DNS in the `iroh.link` domain.
    ///
    /// - It uses an mDNS-like system to announce itself on the local network.
    ///
    /// # Usage during tests
    ///
    /// Note that the default changes when compiling with `cfg(test)` or the `test-utils`
    /// cargo feature from [iroh-net] is enabled.  In this case only the Pkarr/DNS service
    /// is used, but on the `iroh.test` domain.  This domain is not integrated with the
    /// global DNS network and thus node discovery is effectively disabled.  To use node
    /// discovery in a test use the [`iroh_net::test_utils::DnsPkarrServer`] in the test and
    /// configure it here as a custom discovery mechanism ([`DiscoveryConfig::Custom`]).
    ///
    /// [number 0]: https://n0.computer
    #[default]
    Default,
}

/// An Iroh node. Allows you to sync, store, and transfer data.
#[derive(uniffi::Object, Debug, Clone)]
pub struct Iroh {
    router: iroh::protocol::Router,
    _local_pool: Arc<LocalPool>,
    /// RPC client for node and net to hand out
    pub(crate) client: RpcClient<
        iroh_node_util::rpc::proto::RpcService,
        FlumeConnector<iroh_node_util::rpc::proto::Response, iroh_node_util::rpc::proto::Request>,
    >,
    /// Handler task
    _handler: Arc<AbortOnDropHandle<()>>,
    pub(crate) blobs_client: BlobsClient,
    pub(crate) tags_client: TagsClient,
    pub(crate) net_client: NetClient,
    pub(crate) authors_client: Option<AuthorsClient>,
    pub(crate) docs_client: Option<DocsClient>,
    pub(crate) gossip: Gossip,
    pub(crate) node_client: iroh_node_util::rpc::client::node::Client,
}

pub(crate) type NetClient = iroh_node_util::rpc::client::net::Client;
pub(crate) type BlobsClient = iroh_blobs::rpc::client::blobs::Client<
    FlumeConnector<iroh_blobs::rpc::proto::Response, iroh_blobs::rpc::proto::Request>,
>;
pub(crate) type TagsClient = iroh_blobs::rpc::client::tags::Client<
    FlumeConnector<iroh_blobs::rpc::proto::Response, iroh_blobs::rpc::proto::Request>,
>;
pub(crate) type AuthorsClient = iroh_docs::rpc::client::authors::Client<
    FlumeConnector<iroh_docs::rpc::proto::Response, iroh_docs::rpc::proto::Request>,
>;
pub(crate) type DocsClient = iroh_docs::rpc::client::docs::Client<
    FlumeConnector<iroh_docs::rpc::proto::Response, iroh_docs::rpc::proto::Request>,
>;

#[derive(Debug, Clone)]
struct NetNode(iroh::Endpoint);

impl AbstractNode for NetNode {
    fn endpoint(&self) -> &iroh::Endpoint {
        &self.0
    }

    fn shutdown(&self) {}
}

/// An Iroh node builder
#[derive(uniffi::Object, Debug)]
pub struct IrohBuilder {
    // set to `None` after building
    router: Mutex<Option<iroh::protocol::RouterBuilder>>,
}

#[uniffi::export]
impl IrohBuilder {
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn create(options: NodeOptions) -> Result<Self, IrohError> {
        let mut ep_builder = iroh::Endpoint::builder();
        ep_builder = match options.node_discovery {
            Some(NodeDiscoveryConfig::None) => ep_builder.clear_discovery(),
            Some(NodeDiscoveryConfig::Default) | None => ep_builder.discovery_n0(),
        };
        let endpoint = ep_builder.bind().await?;
        let router = iroh::protocol::Router::builder(endpoint);

        Ok(Self {
            router: Mutex::new(Some(router)),
        })
    }

    #[uniffi::method]
    pub fn endpoint(&self) -> Endpoint {
        let ep = self
            .router
            .lock()
            .unwrap()
            .as_ref()
            .expect("already built")
            .endpoint()
            .clone();
        Endpoint::new(ep)
    }

    #[uniffi::method]
    pub fn accept(&self, alpn: &[u8], handler: Arc<dyn ProtocolHandler>) {
        let mut router_lock = self.router.lock().unwrap();
        let mut router = router_lock.take().expect("already built");

        router = router.accept(alpn, ProtocolWrapper { handler });
        router_lock.replace(router);
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn build(&self) -> Result<Iroh, IrohError> {
        let mut router = self.router.lock().unwrap().take().expect("already built");
        todo!()
    }
}

#[uniffi::export]
impl Iroh {
    /// Create a new iroh node.
    ///
    /// The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn persistent(path: String) -> Result<Self, IrohError> {
        let options = NodeOptions::default();
        Self::persistent_with_options(path, options).await
    }

    /// Create a new iroh node.
    ///
    /// All data will be only persistet in memory.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn memory() -> Result<Self, IrohError> {
        let options = NodeOptions::default();
        Self::memory_with_options(options).await
    }

    /// Create a new iroh node with options.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn persistent_with_options(
        path: String,
        options: NodeOptions,
    ) -> Result<Self, IrohError> {
        let path = PathBuf::from(path);
        tokio::fs::create_dir_all(&path)
            .await
            .map_err(|err| anyhow::anyhow!(err))?;

        let builder = iroh::Endpoint::builder();
        let (docs_store, author_store) = if options.enable_docs {
            let docs_store = iroh_docs::store::Store::persistent(path.join("docs.redb"))?;
            let author_store =
                iroh_docs::engine::DefaultAuthorStorage::Persistent(path.join("default-author"));

            (Some(docs_store), Some(author_store))
        } else {
            (None, None)
        };
        let blobs_store = iroh_blobs::store::fs::Store::load(path.join("blobs"))
            .await
            .map_err(|err| anyhow::anyhow!(err))?;
        let local_pool = LocalPool::default();
        let (builder, gossip, blobs, docs) = apply_options(
            builder,
            options,
            blobs_store,
            docs_store,
            author_store,
            &local_pool,
        )
        .await?;
        let router = builder.spawn().await?;

        let (listener, connector) = quic_rpc::transport::flume::channel(1);
        let listener = RpcServer::new(listener);
        let client = RpcClient::new(connector);
        let nn = Arc::new(NetNode(router.endpoint().clone()));
        let handler = listener.spawn_accept_loop(move |req, chan| {
            iroh_node_util::rpc::server::handle_rpc_request(nn.clone(), req, chan)
        });

        let blobs_client = blobs.client().clone();
        let net_client = iroh_node_util::rpc::client::net::Client::new(client.clone().boxed());

        let docs_client = docs.map(|d| d.client().clone());

        let node_client = iroh_node_util::rpc::client::node::Client::new(client.clone().boxed());

        Ok(Iroh {
            router,
            _local_pool: Arc::new(local_pool),
            client,
            _handler: Arc::new(handler),
            tags_client: blobs_client.tags(),
            blobs_client,
            net_client,
            authors_client: docs_client.as_ref().map(|d| d.authors()),
            docs_client,
            gossip,
            node_client,
        })
    }

    /// Create a new in memory iroh node with options.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn memory_with_options(options: NodeOptions) -> Result<Self, IrohError> {
        let builder = iroh::Endpoint::builder();

        let (docs_store, author_store) = if options.enable_docs {
            let docs_store = iroh_docs::store::Store::memory();
            let author_store = iroh_docs::engine::DefaultAuthorStorage::Mem;

            (Some(docs_store), Some(author_store))
        } else {
            (None, None)
        };
        let blobs_store = iroh_blobs::store::mem::Store::default();
        let local_pool = LocalPool::default();
        let (builder, gossip, blobs, docs) = apply_options(
            builder,
            options,
            blobs_store,
            docs_store,
            author_store,
            &local_pool,
        )
        .await?;
        let router = builder.spawn().await?;

        let (listener, connector) = quic_rpc::transport::flume::channel(1);
        let listener = RpcServer::new(listener);
        let client = RpcClient::new(connector);
        let nn: Arc<dyn AbstractNode> = Arc::new(NetNode(router.endpoint().clone()));
        let handler = listener.spawn_accept_loop(move |req, chan| {
            iroh_node_util::rpc::server::handle_rpc_request(nn.clone(), req, chan)
        });

        let blobs_client = blobs.client().clone();
        let net_client = iroh_node_util::rpc::client::net::Client::new(client.clone().boxed());

        let docs_client = docs.map(|d| d.client().clone());

        let node_client = iroh_node_util::rpc::client::node::Client::new(client.clone().boxed());
        Ok(Iroh {
            router,
            _local_pool: Arc::new(local_pool),
            client,
            _handler: Arc::new(handler),
            net_client,
            tags_client: blobs_client.tags(),
            blobs_client,
            authors_client: docs_client.as_ref().map(|d| d.authors()),
            docs_client,
            gossip,
            node_client,
        })
    }

    /// Get statistics of the running node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stats(&self) -> Result<HashMap<String, CounterStats>, IrohError> {
        let stats = self.node_client.stats().await?;
        let stats = stats
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    CounterStats {
                        value: u32::try_from(v.value).expect("value too large"),
                        description: v.description,
                    },
                )
            })
            .collect();
        Ok(stats)
    }

    /// Get status information about a node
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn status(&self) -> Result<Arc<NodeStatus>, IrohError> {
        let res = self
            .node_client
            .status()
            .await
            .map(|n| Arc::new(n.into()))?;
        Ok(res)
    }

    /// Shutdown this iroh node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn shutdown(&self) -> Result<(), IrohError> {
        self.router.shutdown().await?;
        Ok(())
    }

    #[uniffi::method]
    pub fn endpoint(&self) -> Endpoint {
        Endpoint::new(self.router.endpoint().clone())
    }
}

async fn apply_options<S: iroh_blobs::store::Store>(
    mut builder: iroh::endpoint::Builder,
    options: NodeOptions,
    blob_store: S,
    docs_store: Option<iroh_docs::store::Store>,
    author_store: Option<iroh_docs::engine::DefaultAuthorStorage>,
    local_pool: &LocalPool,
) -> anyhow::Result<(
    iroh::protocol::RouterBuilder,
    Gossip,
    Blobs<S>,
    Option<Docs<S>>,
)> {
    let gc_period = if let Some(millis) = options.gc_interval_millis {
        match millis {
            0 => None,
            millis => Some(Duration::from_millis(millis)),
        }
    } else {
        None
    };

    let blob_events = if let Some(blob_events_cb) = options.blob_events {
        BlobProvideEvents::new(blob_events_cb).into()
    } else {
        EventSender::default()
    };

    if let Some(addr) = options.ipv4_addr {
        builder = builder.bind_addr_v4(addr.parse()?);
    }

    if let Some(addr) = options.ipv6_addr {
        builder = builder.bind_addr_v6(addr.parse()?);
    }

    builder = match options.node_discovery {
        Some(NodeDiscoveryConfig::None) => builder.clear_discovery(),
        Some(NodeDiscoveryConfig::Default) | None => builder.discovery_n0(),
    };

    if let Some(secret_key) = options.secret_key {
        let key: [u8; 32] = AsRef::<[u8]>::as_ref(&secret_key).try_into()?;
        let key = iroh::SecretKey::from_bytes(&key);
        builder = builder.secret_key(key);
    }

    let endpoint = builder.bind().await?;
    let mut builder = iroh::protocol::Router::builder(endpoint);

    let endpoint = Arc::new(Endpoint::new(builder.endpoint().clone()));

    // Add default protocols for now

    // iroh gossip
    let gossip = Gossip::builder().spawn(builder.endpoint().clone()).await?;
    builder = builder.accept(iroh_gossip::ALPN, gossip.clone());

    // iroh blobs
    let downloader = Downloader::new(
        blob_store.clone(),
        builder.endpoint().clone(),
        local_pool.handle().clone(),
    );
    let blobs = Blobs::new(
        blob_store.clone(),
        local_pool.handle().clone(),
        blob_events,
        downloader.clone(),
        builder.endpoint().clone(),
    );

    builder = builder.accept(iroh_blobs::ALPN, blobs.clone());

    let docs = if options.enable_docs {
        let engine = iroh_docs::engine::Engine::spawn(
            builder.endpoint().clone(),
            gossip.clone(),
            docs_store.expect("docs enabled"),
            blob_store.clone(),
            downloader,
            author_store.expect("docs enabled"),
            local_pool.handle().clone(),
        )
        .await?;
        let docs = Docs::new(engine);
        builder = builder.accept(iroh_docs::ALPN, docs.clone());
        blobs.add_protected(docs.protect_cb())?;

        Some(docs)
    } else {
        None
    };
    if let Some(period) = gc_period {
        blobs.start_gc(GcConfig {
            period,
            done_callback: None,
        })?;
    }

    // Add custom protocols
    if let Some(protocols) = options.protocols {
        for (alpn, protocol) in protocols {
            let handler = protocol.create(endpoint.clone());
            builder = builder.accept(alpn, ProtocolWrapper { handler });
        }
    }

    Ok((builder, gossip, blobs, docs))
}

/// The response to a status request
#[derive(Debug, uniffi::Object)]
pub struct NodeStatus(iroh_node_util::rpc::client::net::NodeStatus);

impl From<iroh_node_util::rpc::client::net::NodeStatus> for NodeStatus {
    fn from(n: iroh_node_util::rpc::client::net::NodeStatus) -> Self {
        NodeStatus(n)
    }
}

#[uniffi::export]
impl NodeStatus {
    /// The node id and socket addresses of this node.
    pub fn node_addr(&self) -> Arc<NodeAddr> {
        Arc::new(self.0.addr.clone().into())
    }

    /// The bound listening addresses of the node
    pub fn listen_addrs(&self) -> Vec<String> {
        self.0
            .listen_addrs
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    /// The version of the node
    pub fn version(&self) -> String {
        self.0.version.clone()
    }

    /// The address of the RPC of the node
    pub fn rpc_addr(&self) -> Option<String> {
        self.0.rpc_addr.map(|a| a.to_string())
    }
}

#[derive(Clone)]
struct BlobProvideEvents {
    callback: Arc<dyn BlobProvideEventCallback>,
}

impl Debug for BlobProvideEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobProvideEvents()")
    }
}

impl BlobProvideEvents {
    fn new(callback: Arc<dyn BlobProvideEventCallback>) -> Self {
        Self { callback }
    }
}

impl iroh_blobs::provider::CustomEventSender for BlobProvideEvents {
    fn send(&self, event: iroh_blobs::provider::Event) -> futures_lite::future::Boxed<()> {
        let cb = self.callback.clone();
        Box::pin(async move {
            cb.blob_event(Arc::new(event.into())).await.ok();
        })
    }

    fn try_send(&self, event: iroh_blobs::provider::Event) {
        let cb = self.callback.clone();
        tokio::task::spawn(async move {
            cb.blob_event(Arc::new(event.into())).await.ok();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory() {
        let node = Iroh::memory().await.unwrap();
        let id = node.net().node_id().await.unwrap();
        println!("{id}");
    }
}
