use std::{collections::HashMap, path::PathBuf};

use crate::{
    ConnType, ConnectionTypeMixed, CounterStats, DirectAddrInfo, IrohNode, NodeStatusResponse,
    PublicKey,
};

use futures::TryStreamExt;
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

#[napi]
impl DirectAddrInfo {
    /// Get the reported latency, if it exists, in milliseconds
    #[napi(js_name = "latency")]
    pub fn latency_js(&self) -> Option<u128> {
        self.0.latency.map(|l| l.as_millis())
    }

    /// Get the last control message received by this node
    #[napi(js_name = "lastControl")]
    pub fn last_control_js(&self) -> Option<JsLatencyAndControlMsg> {
        self.0
            .last_control
            .map(|(latency, control_msg)| JsLatencyAndControlMsg {
                latency: latency.as_millis().into(),
                control_msg: control_msg.to_string(),
            })
    }

    /// Get how long ago the last payload message was received for this node in milliseconds.
    #[napi(js_name = "lastPayload")]
    pub fn last_payload_js(&self) -> Option<u128> {
        self.0.last_payload.map(|d| d.as_millis())
    }
}

/// The latency and type of the control message
#[napi(constructor, js_name = "LatencyAndControlMsg")]
pub struct JsLatencyAndControlMsg {
    /// The latency of the control message, in milliseconds.
    pub latency: BigInt,
    /// The type of control message, represented as a string
    pub control_msg: String,
}

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
    pub latency: Option<BigInt>,
    /// Duration since the last time this peer was used (in milliseconds).
    pub last_used: Option<BigInt>,
}

#[napi]
impl JsConnectionInfo {
    #[napi]
    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    #[napi]
    pub fn addrs(&self) -> Vec<DirectAddrInfo> {
        self.addrs.clone()
    }
}

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
            latency: value.latency.map(|l| l.as_millis().into()),
            last_used: value.last_used.map(|l| l.as_millis().into()),
        }
    }
}

#[napi(object, js_name = "ConnectionType")]
#[derive(Debug, Clone)]
pub struct JsConnectionType {
    pub typ: ConnType,
    pub data0: Option<String>,
    pub data1: Option<String>,
}

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

#[napi]
impl IrohNode {
    /// Create a new iroh node. The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    #[napi(factory, js_name = "withPath")]
    pub async fn new_js(path: String) -> napi::Result<Self> {
        let res = Self::new_inner(PathBuf::from(path), None).await?;
        Ok(res)
    }

    /// Get statistics of the running node.
    #[napi(js_name = "stats")]
    pub async fn stats_js(&self) -> Result<HashMap<String, CounterStats>, napi::Error> {
        let stats = self.sync_client.node.stats().await?;
        Ok(stats.into_iter().map(|(k, v)| (k, v.into())).collect())
    }

    /// Return `ConnectionInfo`s for each connection we have to another iroh node.
    #[napi(js_name = "connections")]
    pub async fn connections_js(&self) -> Result<Vec<JsConnectionInfo>, napi::Error> {
        let infos = self
            .sync_client
            .node
            .connections()
            .await?
            .map_ok(|info| info.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(infos)
    }

    /// Return connection information on the currently running node.
    #[napi(js_name = "connectionInfo")]
    pub async fn connection_info_js(
        &self,
        node_id: &PublicKey,
    ) -> Result<Option<JsConnectionInfo>, napi::Error> {
        let info = self
            .sync_client
            .node
            .connection_info(node_id.into())
            .await?;

        Ok(info.map(Into::into))
    }

    /// Get status information about a node
    #[napi(js_name = "status")]
    pub async fn status_js(&self) -> Result<NodeStatusResponse, napi::Error> {
        let status = self.sync_client.node.status().await.map(Into::into)?;
        Ok(status)
    }
}
