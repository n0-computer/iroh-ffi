use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc, time::Duration};

use iroh::node::{FsNode, MemNode};

use crate::{BlobProvideEventCallback, IrohError, NodeAddr, PublicKey};

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
pub struct DirectAddrInfo(pub(crate) iroh::net::endpoint::DirectAddrInfo);

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
/// The kinds of control messages that can be sent
// pub use iroh::net::magicsock::ControlMsg;

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

impl From<iroh::net::endpoint::RemoteInfo> for RemoteInfo {
    fn from(value: iroh::net::endpoint::RemoteInfo) -> Self {
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

impl From<iroh::net::endpoint::ConnectionType> for ConnectionType {
    fn from(value: iroh::net::endpoint::ConnectionType) -> Self {
        match value {
            iroh::net::endpoint::ConnectionType::Direct(addr) => {
                ConnectionType::Direct(addr.to_string())
            }
            iroh::net::endpoint::ConnectionType::Mixed(addr, url) => {
                ConnectionType::Mixed(addr.to_string(), url.to_string())
            }
            iroh::net::endpoint::ConnectionType::Relay(url) => {
                ConnectionType::Relay(url.to_string())
            }
            iroh::net::endpoint::ConnectionType::None => ConnectionType::None,
        }
    }
}
/// Options passed to [`IrohNode.new`]. Controls the behaviour of an iroh node.
#[derive(uniffi::Record)]
pub struct NodeOptions {
    /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
    /// Set to 0 to disable gc
    pub gc_interval_millis: Option<u64>,
    /// Provide a callback to hook into events when the blobs component adds and provides blobs.
    pub blob_events: Option<Arc<dyn BlobProvideEventCallback>>,
}

impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            gc_interval_millis: Some(0),
            blob_events: None,
        }
    }
}

impl Debug for NodeOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NodeOptions(gc_interval_millis: {:?}, blob_events: {})",
            self.gc_interval_millis,
            self.blob_events.is_some()
        )
    }
}

/// An Iroh node. Allows you to sync, store, and transfer data.
#[derive(uniffi::Object, Debug, Clone)]
pub enum Iroh {
    Fs(FsNode),
    Memory(MemNode),
}

impl Iroh {
    pub(crate) fn client(&self) -> &iroh::client::Iroh {
        match self {
            Self::Fs(node) => node,
            Self::Memory(node) => node,
        }
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

        let mut builder = iroh::node::Builder::default().persist(path).await?;
        if let Some(millis) = options.gc_interval_millis {
            let policy = match millis {
                0 => iroh::node::GcPolicy::Disabled,
                millis => iroh::node::GcPolicy::Interval(Duration::from_millis(millis)),
            };
            builder = builder.gc_policy(policy);
        }
        if let Some(blob_events_cb) = options.blob_events {
            builder = builder.blobs_events(BlobProvideEvents::new(blob_events_cb))
        }
        let node = builder.spawn().await?;

        Ok(Iroh::Fs(node))
    }

    /// Create a new in memory iroh node with options.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn memory_with_options(options: NodeOptions) -> Result<Self, IrohError> {
        let mut builder = iroh::node::Builder::default();
        if let Some(millis) = options.gc_interval_millis {
            let policy = match millis {
                0 => iroh::node::GcPolicy::Disabled,
                millis => iroh::node::GcPolicy::Interval(Duration::from_millis(millis)),
            };
            builder = builder.gc_policy(policy);
        }
        if let Some(blob_events_cb) = options.blob_events {
            builder = builder.blobs_events(BlobProvideEvents::new(blob_events_cb))
        }
        let node = builder.spawn().await?;

        Ok(Iroh::Memory(node))
    }

    /// Access to node specific funtionaliy.
    pub fn node(&self) -> Node {
        Node { node: self.clone() }
    }
}

/// Iroh node client.
#[derive(uniffi::Object)]
pub struct Node {
    node: Iroh,
}

impl Node {
    fn node(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[uniffi::export]
impl Node {
    /// Get statistics of the running node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stats(&self) -> Result<HashMap<String, CounterStats>, IrohError> {
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
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn status(&self) -> Result<Arc<NodeStatus>, IrohError> {
        let res = self.node().status().await.map(|n| Arc::new(n.into()))?;
        Ok(res)
    }

    /// Shutdown this iroh node.
    pub async fn shutdown(&self, force: bool) -> Result<(), IrohError> {
        self.node().shutdown(force).await?;
        Ok(())
    }

    /// Returns `Some(addr)` if an RPC endpoint is running, `None` otherwise.
    pub fn my_rpc_addr(&self) -> Option<String> {
        let addr = match self.node {
            Iroh::Fs(ref n) => n.my_rpc_addr(),
            Iroh::Memory(ref n) => n.my_rpc_addr(),
        };
        addr.map(|a| a.to_string())
    }
}

/// The response to a status request
#[derive(Debug, uniffi::Object)]
pub struct NodeStatus(iroh::client::NodeStatus);

impl From<iroh::client::NodeStatus> for NodeStatus {
    fn from(n: iroh::client::NodeStatus) -> Self {
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

impl iroh::blobs::provider::CustomEventSender for BlobProvideEvents {
    fn send(&self, event: iroh::blobs::provider::Event) -> futures_lite::future::Boxed<()> {
        let cb = self.callback.clone();
        Box::pin(async move {
            cb.blob_event(Arc::new(event.into())).await.ok();
        })
    }

    fn try_send(&self, event: iroh::blobs::provider::Event) {
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
