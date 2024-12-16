use std::{
    collections::HashMap, future::Future, path::PathBuf, pin::Pin, sync::Arc, time::Duration,
};

use iroh_blobs::{
    downloader::Downloader, net_protocol::Blobs, provider::EventSender, store::GcConfig,
    util::local_pool::LocalPool,
};
use iroh_docs::protocol::Docs;
use iroh_gossip::net::Gossip;
use iroh_node_util::rpc::server::AbstractNode;
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use quic_rpc::{transport::flume::FlumeConnector, RpcClient, RpcServer};
use tokio_util::task::AbortOnDropHandle;
use tracing::warn;

use crate::{BlobProvideEvent, Connecting, CounterStats, Endpoint, NodeAddr};

/// Options passed to [`IrohNode.new`]. Controls the behaviour of an iroh node.#
#[napi(object, object_to_js = false)]
pub struct NodeOptions {
    /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
    /// Set to null to disable gc
    pub gc_interval_millis: Option<u32>,
    /// Provide a callback to hook into events when the blobs component adds and provides blobs.
    pub blob_events: Option<ThreadsafeFunction<BlobProvideEvent, ()>>,
    /// Should docs be enabled? Defaults to `false`.
    pub enable_docs: Option<bool>,
    /// Overwrites the default IPv4 address to bind to
    pub ipv4_addr: Option<String>,
    /// Overwrites the default IPv6 address to bind to
    pub ipv6_addr: Option<String>,
    /// Configure the node discovery.
    pub node_discovery: Option<NodeDiscoveryConfig>,
    /// Provide a specific secret key, identifying this node. Must be 32 bytes long.
    pub secret_key: Option<Vec<u8>>,

    pub protocols: Option<HashMap<Vec<u8>, ThreadsafeFunction<Endpoint, ProtocolHandler>>>,
}

#[derive(derive_more::Debug)]
#[napi(object, object_to_js = false)]
pub struct ProtocolHandler {
    #[debug("accept")]
    pub accept: Arc<ThreadsafeFunction<Connecting, ()>>,
    #[debug("shutdown")]
    pub shutdown: Option<Arc<ThreadsafeFunction<(), ()>>>,
}

impl iroh::protocol::ProtocolHandler for ProtocolHandler {
    fn accept(
        &self,
        conn: iroh::endpoint::Connecting,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        let accept = self.accept.clone();
        Box::pin(async move {
            accept.call_async(Ok(Connecting::new(conn))).await?;
            Ok(())
        })
    }

    fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let shutdown = self.shutdown.clone();
        Box::pin(async move {
            if let Some(ref cb) = shutdown {
                if let Err(err) = cb.call_async(Ok(())).await {
                    warn!("shutdown failed: {:?}", err);
                }
            }
        })
    }
}

impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            gc_interval_millis: None,
            blob_events: None,
            enable_docs: None,
            ipv4_addr: None,
            ipv6_addr: None,
            node_discovery: None,
            secret_key: None,
            protocols: None,
        }
    }
}

#[derive(Debug, Default)]
#[napi(string_enum)]
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
    /// [iroh-net]: crate::net
    #[default]
    Default,
}

/// An Iroh node. Allows you to sync, store, and transfer data.
#[derive(Debug, Clone)]
#[napi]
pub struct Iroh {
    pub(crate) router: iroh::protocol::Router,
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

#[napi]
impl Iroh {
    /// Create a new iroh node.
    ///
    /// The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    #[napi(factory)]
    pub async fn persistent(path: String, opts: Option<NodeOptions>) -> Result<Self> {
        let options = opts.unwrap_or_default();

        let path = PathBuf::from(path);
        tokio::fs::create_dir_all(&path)
            .await
            .map_err(|err| anyhow::anyhow!(err))?;

        let builder = iroh::Endpoint::builder();
        let (docs_store, author_store) = if options.enable_docs.unwrap_or_default() {
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
        })
    }

    /// Create a new iroh node.
    ///
    /// All data will be only persistet in memory.
    #[napi(factory)]
    pub async fn memory(opts: Option<NodeOptions>) -> Result<Self> {
        let options = opts.unwrap_or_default();
        let builder = iroh::Endpoint::builder();

        let (docs_store, author_store) = if options.enable_docs.unwrap_or_default() {
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
        })
    }

    /// Access to node specific funtionaliy.
    #[napi(getter)]
    pub fn node(&self) -> Node {
        let router = self.router.clone();
        let client = self.client.clone().boxed();
        let client = iroh_node_util::rpc::client::node::Client::new(client);
        Node { router, client }
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
            millis => Some(Duration::from_millis(millis as _)),
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

    let endpoint = Endpoint::new(builder.endpoint().clone());

    // Add default protocols for now

    // iroh gossip
    let gossip = iroh_gossip::net::Gossip::builder()
        .spawn(builder.endpoint().clone())
        .await?;

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

    let docs = if options.enable_docs.unwrap_or_default() {
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
            let handler = protocol.call_async(Ok(endpoint.clone())).await?;
            builder = builder.accept(alpn, handler);
        }
    }

    Ok((builder, gossip, blobs, docs))
}

/// Iroh node client.
#[napi]
pub struct Node {
    router: iroh::protocol::Router,
    client: iroh_node_util::rpc::client::node::Client,
}

#[napi]
impl Node {
    /// Get statistics of the running node.
    #[napi]
    pub async fn stats(&self) -> Result<HashMap<String, CounterStats>> {
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

    /// Get status information about a node
    #[napi]
    pub async fn status(&self) -> Result<NodeStatus> {
        let res = self.client.status().await.map(|n| n.into())?;
        Ok(res)
    }

    /// Shutdown this iroh node.
    #[napi]
    pub async fn shutdown(&self) -> Result<()> {
        self.router.shutdown().await?;

        Ok(())
    }

    #[napi]
    pub fn endpoint(&self) -> Endpoint {
        Endpoint::new(self.router.endpoint().clone())
    }
}

/// The response to a status request
#[derive(Debug)]
#[napi(object)]
pub struct NodeStatus {
    /// The node id and socket addresses of this node.
    pub addr: NodeAddr,
    /// The bound listening addresses of the node
    pub listen_addrs: Vec<String>,
    /// The version of the node
    pub version: String,
    /// RPC address, if currently listening.
    pub rpc_addr: Option<String>,
}

impl From<iroh_node_util::rpc::client::net::NodeStatus> for NodeStatus {
    fn from(n: iroh_node_util::rpc::client::net::NodeStatus) -> Self {
        NodeStatus {
            addr: n.addr.into(),
            listen_addrs: n.listen_addrs.iter().map(|addr| addr.to_string()).collect(),
            version: n.version,
            rpc_addr: n.rpc_addr.map(|a| a.to_string()),
        }
    }
}

#[derive(Clone)]
struct BlobProvideEvents {
    callback: Arc<ThreadsafeFunction<BlobProvideEvent, ()>>,
}

impl std::fmt::Debug for BlobProvideEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobProvideEvents()")
    }
}

impl BlobProvideEvents {
    fn new(callback: ThreadsafeFunction<BlobProvideEvent, ()>) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl iroh_blobs::provider::CustomEventSender for BlobProvideEvents {
    fn send(&self, event: iroh_blobs::provider::Event) -> futures::future::BoxFuture<'static, ()> {
        let cb = self.callback.clone();
        Box::pin(async move {
            let msg = BlobProvideEvent::convert(event);
            if let Err(err) = cb.call_async(msg).await {
                eprintln!("failed call: {:?}", err);
            }
        })
    }

    fn try_send(&self, event: iroh_blobs::provider::Event) {
        let cb = self.callback.clone();
        let msg = BlobProvideEvent::convert(event);
        cb.call(msg, ThreadsafeFunctionCallMode::NonBlocking);
    }
}
