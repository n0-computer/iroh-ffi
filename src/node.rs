use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use futures::stream::TryStreamExt;
use iroh::{
    client::MemIroh,
    node::{Builder, FsNode},
};

use crate::{block_on, IrohError, NodeAddr, PublicKey};

/// Stats counter
/// Counter stats
#[derive(Debug)]
pub struct CounterStats {
    /// The counter value
    pub value: u32,
    /// The counter description
    pub description: String,
}

/// Information about a direct address.
#[derive(Debug, Clone)]
pub struct DirectAddrInfo(pub(crate) iroh::net::endpoint::DirectAddrInfo);

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

/// Information about a connection
#[derive(Debug)]
pub struct ConnectionInfo {
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

impl From<iroh::net::endpoint::ConnectionInfo> for ConnectionInfo {
    fn from(value: iroh::net::endpoint::ConnectionInfo) -> Self {
        ConnectionInfo {
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
#[derive(Debug)]
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
#[derive(Debug)]
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
pub struct NodeOptions {
    /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
    /// Set to 0 to disable gc
    pub gc_interval_millis: Option<u64>,
}

impl From<NodeOptions> for iroh::node::Builder<iroh::blobs::store::mem::Store> {
    fn from(value: NodeOptions) -> Self {
        let mut b = Builder::default();

        if let Some(millis) = value.gc_interval_millis {
            b = match millis {
                0 => b.gc_policy(iroh::node::GcPolicy::Disabled),
                millis => b.gc_policy(iroh::node::GcPolicy::Interval(Duration::from_millis(
                    millis,
                ))),
            };
        }

        b
    }
}

impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            gc_interval_millis: Some(0),
        }
    }
}

/// An Iroh node. Allows you to sync, store, and transfer data.
pub struct IrohNode {
    pub(crate) node: FsNode,
    pub(crate) sync_client: MemIroh,
    #[allow(dead_code)]
    pub(crate) tokio_rt: Option<tokio::runtime::Runtime>,
}

impl IrohNode {
    pub(crate) fn rt(&self) -> tokio::runtime::Handle {
        match self.tokio_rt {
            Some(ref rt) => rt.handle().clone(),
            None => tokio::runtime::Handle::current(),
        }
    }

    /// Create a new iroh node. The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    pub fn new(path: String) -> Result<Self, IrohError> {
        let options = NodeOptions::default();
        Self::with_options(path, options)
    }

    /// Create a new iroh node with options.
    pub fn with_options(path: String, options: NodeOptions) -> Result<Self, IrohError> {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(anyhow::Error::from)?;
        let rt = tokio_rt.handle().clone();

        let path = PathBuf::from(path);
        let node = block_on(&rt, async move {
            Self::new_inner(path, options, Some(tokio_rt)).await
        })?;

        Ok(node)
    }

    pub(crate) async fn new_inner(
        path: PathBuf,
        options: NodeOptions,
        tokio_rt: Option<tokio::runtime::Runtime>,
    ) -> Result<Self, anyhow::Error> {
        let builder: Builder<iroh::blobs::store::mem::Store> = options.into();
        let node = builder.persist(path).await?.spawn().await?;
        let sync_client = node.clone().client().clone();

        Ok(IrohNode {
            node,
            sync_client,
            tokio_rt,
        })
    }

    /// The string representation of the PublicKey of this node.
    pub fn node_id(&self) -> String {
        self.node.node_id().to_string()
    }

    /// Get statistics of the running node.
    pub fn stats(&self) -> Result<HashMap<String, CounterStats>, IrohError> {
        block_on(&self.rt(), async {
            let stats = self.sync_client.stats().await?;
            Ok(stats
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
                .collect())
        })
    }

    /// Return `ConnectionInfo`s for each connection we have to another iroh node.
    pub fn connections(&self) -> Result<Vec<ConnectionInfo>, IrohError> {
        block_on(&self.rt(), async {
            let infos = self
                .sync_client
                .connections()
                .await?
                .map_ok(|info| info.into())
                .try_collect::<Vec<_>>()
                .await?;
            Ok(infos)
        })
    }

    /// Return connection information on the currently running node.
    pub fn connection_info(
        &self,
        node_id: &PublicKey,
    ) -> Result<Option<ConnectionInfo>, IrohError> {
        block_on(&self.rt(), async {
            let info = self
                .sync_client
                .connection_info(node_id.into())
                .await
                .map(|i| i.map(|i| i.into()))?;
            Ok(info)
        })
    }

    /// Get status information about a node
    pub fn status(&self) -> Result<Arc<NodeStatus>, IrohError> {
        block_on(&self.rt(), async {
            let res = self
                .sync_client
                .status()
                .await
                .map(|n| Arc::new(n.into()))?;
            Ok(res)
        })
    }
}

/// The response to a status request
#[derive(Debug)]
pub struct NodeStatus(iroh::client::NodeStatus);

impl From<iroh::client::NodeStatus> for NodeStatus {
    fn from(n: iroh::client::NodeStatus) -> Self {
        NodeStatus(n)
    }
}

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
}
