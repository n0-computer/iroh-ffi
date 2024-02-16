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
pub struct DirectAddrInfo(iroh::net::magicsock::DirectAddrInfo);

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

    /// Get the reported latency, if it exists, in milliseconds
    #[napi(js_name = "latency")]
    pub fn latency_js(&self) -> Option<u32> {
        self.0.latency.map(|l| l.as_millis() as _)
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

    /// Get the last control message received by this node
    #[cfg(feature = "napi")]
    #[napi(js_name = "lastControl")]
    pub fn last_control_js(&self) -> Option<JsLatencyAndControlMsg> {
        self.0
            .last_control
            .map(|(latency, control_msg)| JsLatencyAndControlMsg {
                latency: latency.as_millis() as _,
                control_msg: control_msg.to_string(),
            })
    }

    /// Get how long ago the last payload message was received for this node
    pub fn last_payload(&self) -> Option<Duration> {
        self.0.last_payload
    }

    /// Get how long ago the last payload message was received for this node in milliseconds.
    #[napi(js_name = "lastPayload")]
    pub fn last_payload_js(&self) -> Option<u32> {
        self.0.last_payload.map(|d| d.as_millis() as _)
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

/// The latency and type of the control message
#[cfg(feature = "napi")]
#[napi(constructor, js_name = "LatencyAndControlMsg")]
pub struct JsLatencyAndControlMsg {
    /// The latency of the control message, in milliseconds.
    pub latency: u32,
    /// The type of control message, represented as a string
    pub control_msg: String,
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

#[cfg(feature = "napi")]
#[napi(js_name = "ConnectionInfo")]
#[derive(Debug, Clone)]
pub struct JsConnectionInfo {
    /// The public key of the endpoint.
    public_key: PublicKey,
    /// Derp url, if available.
    pub derp_url: Option<String>,
    /// List of addresses at which this node might be reachable, plus any latency information we
    /// have about that address and the last time the address was used.
    addrs: Vec<DirectAddrInfo>,
    /// The type of connection we have to the peer, either direct or over relay.
    pub conn_type: JsConnectionType,
    /// The latency of the `conn_type` (in milliseconds).
    pub latency: Option<u32>,
    /// Duration since the last time this peer was used (in milliseconds).
    pub last_used: Option<u32>,
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

#[cfg(feature = "napi")]
impl From<iroh::net::magic_endpoint::ConnectionInfo> for JsConnectionInfo {
    fn from(value: iroh::net::magic_endpoint::ConnectionInfo) -> Self {
        Self {
            public_key: value.public_key.into(),
            derp_url: value.derp_url.map(|url| url.to_string()),
            addrs: value
                .addrs
                .iter()
                .map(|a| DirectAddrInfo(a.clone()))
                .collect(),
            conn_type: value.conn_type.into(),
            latency: value.latency.map(|l| l.as_millis() as _),
            last_used: value.last_used.map(|l| l.as_millis() as _),
        }
    }
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

#[cfg(feature = "napi")]
#[napi(object, js_name = "ConnectionType")]
#[derive(Debug, Clone)]
pub struct JsConnectionType {
    pub typ: ConnType,
    pub data0: Option<String>,
    pub data1: Option<String>,
}

#[cfg(feature = "napi")]
impl From<iroh::net::magicsock::ConnectionType> for JsConnectionType {
    fn from(value: iroh::net::magicsock::ConnectionType) -> Self {
        match value {
            iroh::net::magicsock::ConnectionType::Direct(addr) => Self {
                typ: ConnType::Direct,
                data0: Some(addr.to_string()),
                data1: None,
            },
            iroh::net::magicsock::ConnectionType::Mixed(addr, url) => Self {
                typ: ConnType::Mixed,
                data0: Some(addr.to_string()),
                data1: Some(url.to_string()),
            },
            iroh::net::magicsock::ConnectionType::Relay(url) => Self {
                typ: ConnType::Relay,
                data0: Some(url.to_string()),
                data1: None,
            },
            iroh::net::magicsock::ConnectionType::None => Self {
                typ: ConnType::None,
                data0: None,
                data1: None,
            },
        }
    }
}

#[cfg(feature = "napi")]
#[napi]
impl JsConnectionType {
    /// Whether connection is direct, relay, mixed, or none
    #[napi(js_name = "type")]
    pub fn typ(&self) -> ConnType {
        self.typ
    }

    /// Return the socket address if this is a direct connection
    #[napi]
    pub fn as_direct(&self) -> String {
        match self.typ {
            ConnType::Direct => self.data0.as_ref().unwrap().clone(),
            _ => panic!("ConnectionType type is not 'Direct'"),
        }
    }

    /// Return the derp url if this is a relay connection
    #[napi]
    pub fn as_relay(&self) -> String {
        match self.typ {
            ConnType::Relay => self.data0.as_ref().unwrap().clone(),
            _ => panic!("ConnectionType is not `Relay`"),
        }
    }

    /// Return the socket address and DERP url if this is a mixed connection
    #[napi]
    pub fn as_mixed(&self) -> ConnectionTypeMixed {
        match self.typ {
            ConnType::Mixed => ConnectionTypeMixed {
                addr: self.data0.as_ref().unwrap().clone(),
                derp_url: self.data1.as_ref().unwrap().clone(),
            },
            _ => panic!("ConnectionType is not `Mixed`"),
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
    pub(crate) async_runtime: tokio::runtime::Handle,
    pub(crate) sync_client: iroh::client::Iroh<FlumeConnection<ProviderResponse, ProviderRequest>>,
    #[allow(dead_code)]
    pub(crate) tokio_rt: tokio::runtime::Runtime,
}

#[napi]
impl IrohNode {
    /// Create a new iroh node. The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    pub fn new(path: String) -> Result<Self, IrohError> {
        let path = PathBuf::from(path);
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(IrohError::runtime)?;
        let rt = tokio_rt.handle().clone();

        let node = block_on(&rt, async move {
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

            Node::builder(db, docs).secret_key(secret_key).spawn().await
        })
        .map_err(IrohError::node_create)?;

        let sync_client = node.client();

        Ok(IrohNode {
            node,
            async_runtime: rt,
            sync_client,
            tokio_rt,
        })
    }

    /// Create a new iroh node. The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    #[cfg(feature = "napi")]
    #[napi(factory, js_name = "withPath")]
    pub fn new_js(path: String) -> napi::Result<Self> {
        Self::new(path).map_err(|e| napi::Error::from(anyhow::Error::from(e)))
    }

    /// The string representation of the PublicKey of this node.
    #[napi]
    pub fn node_id(&self) -> String {
        self.node.node_id().to_string()
    }

    /// Get statistics of the running node.
    #[napi]
    pub fn stats(&self) -> Result<HashMap<String, CounterStats>, IrohError> {
        block_on(&self.async_runtime, async {
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
        block_on(&self.async_runtime, async {
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

    /// Return `ConnectionInfo`s for each connection we have to another iroh node.
    #[cfg(feature = "napi")]
    #[napi(js_name = "connections")]
    pub fn connections_js(&self) -> Result<Vec<JsConnectionInfo>, IrohError> {
        block_on(&self.async_runtime, async {
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
        block_on(&self.async_runtime, async {
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

    /// Return connection information on the currently running node.
    #[cfg(feature = "napi")]
    #[napi(js_name = "connectionInfo")]
    pub fn connection_info_js(
        &self,
        node_id: &PublicKey,
    ) -> Result<Option<JsConnectionInfo>, IrohError> {
        block_on(&self.async_runtime, async {
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
        block_on(&self.async_runtime, async {
            self.sync_client
                .node
                .status()
                .await
                .map(|n| Arc::new(n.into()))
                .map_err(IrohError::connection)
        })
    }

    /// Get status information about a node
    #[cfg(feature = "napi")]
    #[napi(js_name = "status")]
    pub fn status_js(&self) -> Result<NodeStatusResponse, IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .node
                .status()
                .await
                .map(Into::into)
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
