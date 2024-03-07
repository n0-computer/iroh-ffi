use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use futures::stream::TryStreamExt;
use iroh::{
    node::Node,
    rpc_protocol::{ProviderRequest, ProviderResponse},
};
use napi_derive::napi;
use quic_rpc::transport::flume::FlumeConnection;

use crate::{block_on, IrohError, NodeAddr, PublicKey};

/// Stats counter
/// Counter stats
#[napi(object)]
#[derive(Debug)]
pub struct CounterStats {
    /// The counter value
    pub value: u32,
    /// The counter description
    pub description: String,
}

impl From<iroh::rpc_protocol::CounterStats> for CounterStats {
    fn from(stats: iroh::rpc_protocol::CounterStats) -> Self {
        CounterStats {
            value: stats.value as _,
            description: stats.description,
        }
    }
}

/// Information about a direct address.
#[napi]
#[derive(Debug, Clone)]
pub struct DirectAddrInfo(pub(crate) iroh::net::magicsock::DirectAddrInfo);

#[napi]
impl DirectAddrInfo {
    /// Get the reported address
    #[napi]
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
    /// The public key of the endpoint.
    pub public_key: Arc<PublicKey>,
    /// Derp url, if available.
    pub derp_url: Option<String>,
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

impl From<iroh::net::magic_endpoint::ConnectionInfo> for ConnectionInfo {
    fn from(value: iroh::net::magic_endpoint::ConnectionInfo) -> Self {
        ConnectionInfo {
            public_key: Arc::new(value.public_key.into()),
            derp_url: value.derp_url.map(|url| url.to_string()),
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
#[napi(string_enum)]
#[derive(Debug)]
pub enum ConnType {
    /// Indicates you have a UDP connection.
    Direct,
    /// Indicates you have a DERP relay connection.
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
    /// Relay connection over DERP
    Relay(String),
    /// Both a UDP and a DERP connection are used.
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
                derp_url: url.clone(),
            },
            _ => panic!("ConnectionType is not `Relay`"),
        }
    }
}

/// The socket address and url of the mixed connection
#[napi(constructor)]
pub struct ConnectionTypeMixed {
    /// Address of the node
    pub addr: String,
    /// Url of the DERP node to which the node is connected
    pub derp_url: String,
}

impl From<iroh::net::magicsock::ConnectionType> for ConnectionType {
    fn from(value: iroh::net::magicsock::ConnectionType) -> Self {
        match value {
            iroh::net::magicsock::ConnectionType::Direct(addr) => {
                ConnectionType::Direct(addr.to_string())
            }
            iroh::net::magicsock::ConnectionType::Mixed(addr, url) => {
                ConnectionType::Mixed(addr.to_string(), url.to_string())
            }
            iroh::net::magicsock::ConnectionType::Relay(url) => {
                ConnectionType::Relay(url.to_string())
            }
            iroh::net::magicsock::ConnectionType::None => ConnectionType::None,
        }
    }
}

/// An Iroh node. Allows you to sync, store, and transfer data.
#[napi]
pub struct IrohNode {
    pub(crate) node: Node<iroh::bytes::store::flat::Store>,
    pub(crate) sync_client: iroh::client::Iroh<FlumeConnection<ProviderResponse, ProviderRequest>>,
    #[allow(dead_code)]
    pub(crate) tokio_rt: Option<tokio::runtime::Runtime>,
}

#[napi]
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
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(IrohError::runtime)?;
        let rt = tokio_rt.handle().clone();

        let path = PathBuf::from(path);
        let node = block_on(&rt, async move {
            Self::new_inner(path, Some(tokio_rt))
                .await
                .map_err(IrohError::node_create)
        })?;

        Ok(node)
    }

    pub(crate) async fn new_inner(
        path: PathBuf,
        tokio_rt: Option<tokio::runtime::Runtime>,
    ) -> Result<Self, anyhow::Error> {
        tokio::fs::create_dir_all(&path).await?;
        // create or load secret key
        let secret_key_path = iroh::util::path::IrohPaths::SecretKey.with_root(&path);
        let secret_key = iroh::util::fs::load_secret_key(secret_key_path).await?;

        let docs_path = iroh::util::path::IrohPaths::DocsDatabase.with_root(&path);
        let docs = iroh::sync::store::fs::Store::new(&docs_path)?;

        // create a bao store for the iroh-bytes blobs
        let blob_path = iroh::util::path::IrohPaths::BaoFlatStoreDir.with_root(&path);
        tokio::fs::create_dir_all(&blob_path).await?;
        let db = iroh::bytes::store::flat::Store::load(&blob_path).await?;

        let node = Node::builder(db, docs)
            .secret_key(secret_key)
            .spawn()
            .await?;
        let sync_client = node.client();

        Ok(IrohNode {
            node,
            sync_client,
            tokio_rt,
        })
    }

    /// The string representation of the PublicKey of this node.
    #[napi]
    pub fn node_id(&self) -> String {
        self.node.node_id().to_string()
    }

    /// Get statistics of the running node.
    pub fn stats(&self) -> Result<HashMap<String, CounterStats>, IrohError> {
        block_on(&self.rt(), async {
            let stats = self
                .sync_client
                .node
                .stats()
                .await
                .map_err(IrohError::doc)?;
            Ok(stats.into_iter().map(|(k, v)| (k, v.into())).collect())
        })
    }

    /// Return `ConnectionInfo`s for each connection we have to another iroh node.
    pub fn connections(&self) -> Result<Vec<ConnectionInfo>, IrohError> {
        block_on(&self.rt(), async {
            let infos = self
                .sync_client
                .node
                .connections()
                .await
                .map_err(IrohError::connection)?
                .map_ok(|info| info.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::connection)?;
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
                .node
                .connection_info(node_id.into())
                .await
                .map(|i| i.map(|i| i.into()))
                .map_err(IrohError::connection)?;
            Ok(info)
        })
    }

    /// Get status information about a node
    pub fn status(&self) -> Result<Arc<NodeStatusResponse>, IrohError> {
        block_on(&self.rt(), async {
            self.sync_client
                .node
                .status()
                .await
                .map(|n| Arc::new(n.into()))
                .map_err(IrohError::connection)
        })
    }
}

/// The response to a status request
#[napi]
#[derive(Debug)]
pub struct NodeStatusResponse(iroh::rpc_protocol::NodeStatusResponse);

impl From<iroh::rpc_protocol::NodeStatusResponse> for NodeStatusResponse {
    fn from(n: iroh::rpc_protocol::NodeStatusResponse) -> Self {
        NodeStatusResponse(n)
    }
}

#[napi]
impl NodeStatusResponse {
    /// The node id and socket addresses of this node.
    #[napi]
    pub fn node_addr(&self) -> Arc<NodeAddr> {
        Arc::new(self.0.addr.clone().into())
    }

    /// The bound listening addresses of the node
    #[napi]
    pub fn listen_addrs(&self) -> Vec<String> {
        self.0
            .listen_addrs
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    /// The version of the node
    #[napi]
    pub fn version(&self) -> String {
        self.0.version.clone()
    }
}
