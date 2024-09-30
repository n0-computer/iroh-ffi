use std::{
    collections::HashMap, future::Future, path::PathBuf, pin::Pin, sync::Arc, time::Duration,
};

use iroh::node::{FsNode, MemNode, DEFAULT_RPC_ADDR};
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
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
    /// Enable RPC. Defaults to `false`.
    pub enable_rpc: Option<bool>,
    /// Overwrite the default RPC address.
    pub rpc_addr: Option<String>,
    /// Configure the node discovery.
    pub node_discovery: Option<NodeDiscoveryConfig>,
    /// Provide a specific secret key, identifying this node. Must be 32 bytes long.
    pub secret_key: Option<Vec<u8>>,

    pub protocols: Option<HashMap<Vec<u8>, ThreadsafeFunction<(Endpoint, Iroh), ProtocolHandler>>>,
}

#[derive(derive_more::Debug)]
#[napi(object, object_to_js = false)]
pub struct ProtocolHandler {
    #[debug("accept")]
    pub accept: ThreadsafeFunction<Connecting, ()>,
    #[debug("shutdown")]
    pub shutdown: Option<ThreadsafeFunction<(), ()>>,
}

impl iroh::node::ProtocolHandler for ProtocolHandler {
    fn accept(
        self: Arc<Self>,
        conn: iroh::net::endpoint::Connecting,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            self.accept.call_async(Ok(Connecting::new(conn))).await?;
            Ok(())
        })
    }

    fn shutdown(self: Arc<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            if let Some(ref cb) = self.shutdown {
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
            enable_rpc: None,
            rpc_addr: None,
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
pub struct Iroh(InnerIroh);

#[derive(Debug, Clone)]
enum InnerIroh {
    Fs(FsNode),
    Memory(MemNode),
    Client(iroh::client::Iroh),
}

impl Iroh {
    pub(crate) fn inner_client(&self) -> &iroh::client::Iroh {
        match &self.0 {
            InnerIroh::Fs(node) => node,
            InnerIroh::Memory(node) => node,
            InnerIroh::Client(client) => client,
        }
    }
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

        let builder = iroh::node::Builder::default().persist(path).await?;
        let builder = apply_options(builder, options).await?;
        let node = builder.spawn().await?;

        Ok(Iroh(InnerIroh::Fs(node)))
    }

    /// Create a new iroh node.
    ///
    /// All data will be only persistet in memory.
    #[napi(factory)]
    pub async fn memory(opts: Option<NodeOptions>) -> Result<Self> {
        let options = opts.unwrap_or_default();

        let builder = iroh::node::Builder::default();
        let builder = apply_options(builder, options).await?;
        let node = builder.spawn().await?;

        Ok(Iroh(InnerIroh::Memory(node)))
    }

    /// Create a new iroh client, connecting to an existing node.
    #[napi(factory)]
    pub async fn client(addr: Option<String>) -> Result<Self> {
        let addr = match addr {
            Some(addr) => addr.parse().map_err(anyhow::Error::from)?,
            None => DEFAULT_RPC_ADDR,
        };
        let client = iroh::client::Iroh::connect_addr(addr).await?;

        Ok(Iroh(InnerIroh::Client(client)))
    }

    /// Access to node specific funtionaliy.
    #[napi(getter)]
    pub fn node(&self) -> Node {
        Node { node: self.clone() }
    }
}

async fn apply_options<S: iroh::blobs::store::Store>(
    mut builder: iroh::node::Builder<S>,
    options: NodeOptions,
) -> Result<iroh::node::ProtocolBuilder<S>> {
    if let Some(millis) = options.gc_interval_millis {
        let policy = match millis {
            0 => iroh::node::GcPolicy::Disabled,
            millis => iroh::node::GcPolicy::Interval(Duration::from_millis(millis as _)),
        };
        builder = builder.gc_policy(policy);
    }

    if let Some(blob_events_cb) = options.blob_events {
        builder = builder.blobs_events(BlobProvideEvents::new(blob_events_cb))
    }

    if options.enable_docs.unwrap_or(false) {
        builder = builder.enable_docs();
    }

    if let Some(addr) = options.ipv4_addr {
        builder = builder.bind_addr_v4(addr.parse().map_err(anyhow::Error::from)?);
    }

    if let Some(addr) = options.ipv6_addr {
        builder = builder.bind_addr_v6(addr.parse().map_err(anyhow::Error::from)?);
    }

    if options.enable_rpc.unwrap_or(false) {
        builder = builder.enable_rpc().await?;
    }

    if let Some(addr) = options.rpc_addr {
        builder = builder
            .enable_rpc_with_addr(addr.parse().map_err(anyhow::Error::from)?)
            .await?;
    }
    builder = match options.node_discovery {
        Some(NodeDiscoveryConfig::None) => {
            builder.node_discovery(iroh::node::DiscoveryConfig::None)
        }
        Some(NodeDiscoveryConfig::Default) | None => {
            builder.node_discovery(iroh::node::DiscoveryConfig::Default)
        }
    };

    if let Some(secret_key) = options.secret_key {
        let key: [u8; 32] = AsRef::<[u8]>::as_ref(&secret_key)
            .try_into()
            .map_err(anyhow::Error::from)?;
        let key = iroh::net::key::SecretKey::from_bytes(&key);
        builder = builder.secret_key(key);
    }

    let mut builder = builder.build().await?;
    if let Some(protocols) = options.protocols {
        let endpoint = Endpoint::new(builder.endpoint().clone());
        let client = Iroh(InnerIroh::Client(builder.client().clone()));
        for (alpn, protocol) in protocols {
            let handler = protocol
                .call_async(Ok((endpoint.clone(), client.clone())))
                .await?;
            builder = builder.accept(alpn, Arc::new(handler));
        }
    }

    Ok(builder)
}

/// Iroh node client.
#[napi]
pub struct Node {
    node: Iroh,
}

impl Node {
    fn node(&self) -> &iroh::client::Iroh {
        self.node.inner_client()
    }
}

#[napi]
impl Node {
    /// Get statistics of the running node.
    #[napi]
    pub async fn stats(&self) -> Result<HashMap<String, CounterStats>> {
        let stats = self.node().stats().await?;
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
        let res = self.node().status().await.map(|n| n.into())?;
        Ok(res)
    }

    /// Shutdown this iroh node.
    #[napi]
    pub async fn shutdown(&self, force: bool) -> Result<()> {
        self.node().shutdown(force).await?;
        Ok(())
    }

    /// Returns `Some(addr)` if an RPC endpoint is running, `None` otherwise.
    #[napi]
    pub fn my_rpc_addr(&self) -> Option<String> {
        let addr = match self.node.0 {
            InnerIroh::Fs(ref n) => n.my_rpc_addr(),
            InnerIroh::Memory(ref n) => n.my_rpc_addr(),
            InnerIroh::Client(_) => None, // Not yet available
        };
        addr.map(|a| a.to_string())
    }

    #[napi]
    pub fn endpoint(&self) -> Option<Endpoint> {
        match self.node.0 {
            InnerIroh::Fs(ref n) => Some(Endpoint::new(n.endpoint().clone())),
            InnerIroh::Memory(ref n) => Some(Endpoint::new(n.endpoint().clone())),
            InnerIroh::Client(_) => None, // Not yet available
        }
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

impl From<iroh::client::NodeStatus> for NodeStatus {
    fn from(n: iroh::client::NodeStatus) -> Self {
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
    callback: ThreadsafeFunction<BlobProvideEvent, ()>,
}

impl std::fmt::Debug for BlobProvideEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobProvideEvents()")
    }
}

impl BlobProvideEvents {
    fn new(callback: ThreadsafeFunction<BlobProvideEvent, ()>) -> Self {
        Self { callback }
    }
}

impl iroh::blobs::provider::CustomEventSender for BlobProvideEvents {
    fn send(&self, event: iroh::blobs::provider::Event) -> futures::future::BoxFuture<'static, ()> {
        let cb = self.callback.clone();
        Box::pin(async move {
            let msg = BlobProvideEvent::convert(event);
            if let Err(err) = cb.call_async(msg).await {
                eprintln!("failed call: {:?}", err);
            }
        })
    }

    fn try_send(&self, event: iroh::blobs::provider::Event) {
        let cb = self.callback.clone();
        let msg = BlobProvideEvent::convert(event);
        cb.call(msg, ThreadsafeFunctionCallMode::NonBlocking);
    }
}
