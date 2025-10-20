use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc, time::Duration};

use iroh::protocol::AcceptError;
use iroh_blobs::BlobsProtocol as Blobs;
use iroh_blobs::store::fs::options::GcConfig;
use iroh_docs::protocol::Docs;
use iroh_gossip::net::Gossip;

use crate::{
    BlobProvideEventCallback, CallbackError, Connection, Endpoint, IrohError, NodeAddr, PublicKey,
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
            .map_err(|err| AcceptError::from_err(err))?;
        Ok(())
    }

    async fn shutdown(&self) {
        let this = self.clone();
        this.handler.shutdown().await;
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
    pub(crate) blobs_client: BlobsClient,
    pub(crate) tags_client: TagsClient,
    pub(crate) docs_client: Option<DocsClient>,
    pub(crate) gossip: Gossip,
}

pub(crate) type BlobsClient = iroh_blobs::api::blobs::Blobs;
pub(crate) type TagsClient = iroh_blobs::api::tags::Tags;
pub(crate) type DocsClient = iroh_docs::api::DocsApi;

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
        let blobs_store = iroh_blobs::store::fs::FsStore::load(path.join("blobs"))
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
        let router = builder.spawn();

        let blobs_client = blobs.client().clone();
        let docs_client = docs.map(|d| d.client().clone());

        Ok(Iroh {
            router,
            tags_client: blobs_client.tags(),
            blobs_client,
            docs_client,
            gossip,
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
        let blobs_store = iroh_blobs::store::mem::MemStore::default();
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
        let router = builder.spawn();

        let blobs_client = blobs.client().clone();
        let docs_client = docs.map(|d| d.client().clone());

        Ok(Iroh {
            router,
            tags_client: blobs_client.tags(),
            blobs_client,
            docs_client,
            gossip,
        })
    }

    /// Access to node specific funtionaliy.
    pub fn node(&self) -> Node {
        let router = self.router.clone();
        Node { router }
    }
}

async fn apply_options(
    mut builder: iroh::endpoint::Builder,
    options: NodeOptions,
    blobs_store: &iroh_blobs::api::Store,
    docs_store: Option<iroh_docs::store::Store>,
    author_store: Option<iroh_docs::engine::DefaultAuthorStorage>,
) -> anyhow::Result<(iroh::protocol::RouterBuilder, Gossip, Blobs, Option<Docs>)> {
    let gc_period = if let Some(millis) = options.gc_interval_millis {
        match millis {
            0 => None,
            millis => Some(Duration::from_millis(millis)),
        }
    } else {
        None
    };

    let blob_events = if let Some(blob_events_cb) = options.blob_events {
        Some(BlobProvideEvents::new(blob_events_cb).into())
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

    let blobs = Blobs::new(blobs_store, blob_events);
    let downloader = blobs.downloader(&endpoint);

    builder = builder.accept(iroh_blobs::ALPN, blobs.clone());

    let docs = if options.enable_docs {
        let engine = iroh_docs::engine::Engine::spawn(
            builder.endpoint().clone(),
            gossip.clone(),
            docs_store.expect("docs enabled"),
            (*blobs).clone(),
            downloader,
            author_store.expect("docs enabled"),
            None,
        )
        .await?;
        let docs = Docs::new(engine);
        blobs.add_protected(docs.protect_cb())?;
        builder = builder.accept(iroh_docs::ALPN, docs.clone());

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

/// Iroh node client.
#[derive(uniffi::Object)]
pub struct Node {
    router: iroh::protocol::Router,
}

#[uniffi::export]
impl Node {
    /// Get statistics of the running node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stats(&self) -> Result<HashMap<String, CounterStats>, IrohError> {
        let stats = self.client.stats().await?;
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

impl Node {
    pub(crate) fn raw_endpoint(&self) -> &iroh::Endpoint {
        self.router.endpoint()
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
