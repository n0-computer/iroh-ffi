use futures::TryStreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{Iroh, PublicKey};

/// Iroh net client.
#[napi]
pub struct Net {
    node: Iroh,
}

#[napi]
impl Iroh {
    /// Access to net specific funtionaliy.
    #[napi(getter)]
    pub fn net(&self) -> Net {
        Net { node: self.clone() }
    }
}

impl Net {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[napi]
impl Net {
    /// Return `RemoteInfo`s for nodes we know about.
    #[napi]
    pub async fn remote_info_list(&self) -> Result<Vec<RemoteInfo>> {
        let infos = self
            .client()
            .net()
            .remote_info_iter()
            .await?
            .map_ok(|info| info.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(infos)
    }

    /// Return information on the given remote node.
    #[napi]
    pub async fn remote_info(&self, node_id: &PublicKey) -> Result<Option<RemoteInfo>> {
        let info = self
            .client()
            .net()
            .remote_info(node_id.into())
            .await
            .map(|i| i.map(|i| i.into()))?;
        Ok(info)
    }

    /// The string representation of the PublicKey of this node.
    #[napi]
    pub async fn node_id(&self) -> Result<String> {
        let id = self.client().net().node_id().await?;
        Ok(id.to_string())
    }

    /// Return the [`NodeAddr`] for this node.
    #[napi]
    pub async fn node_addr(&self) -> Result<NodeAddr> {
        let addr = self.client().net().node_addr().await?;
        Ok(addr.into())
    }

    /// Add a known node address to the node.
    #[napi]
    pub async fn add_node_addr(&self, addr: NodeAddr) -> Result<()> {
        self.client()
            .net()
            .add_node_addr(addr.clone().try_into()?)
            .await?;
        Ok(())
    }

    /// Get the relay server we are connected to.
    #[napi]
    pub async fn home_relay(&self) -> Result<Option<String>> {
        let relay = self.client().net().home_relay().await?;
        Ok(relay.map(|u| u.to_string()))
    }
}

/// Stats counter
#[derive(Debug)]
#[napi(object)]
pub struct CounterStats {
    /// The counter value
    pub value: u32,
    /// The counter description
    pub description: String,
}

/// Information about a direct address.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DirectAddrInfo {
    /// The address reported.
    pub addr: String,
    /// The latency to the address, if any.
    pub latency: Option<u32>,
    /// Last control message received by this node.
    pub last_control_time: Option<u32>,
    pub last_control_msg: Option<String>,
    /// How long ago was the last payload message for this node.
    pub last_payload: Option<u32>,
    /// When was this connection last alive, if ever.
    pub last_alive: Option<u32>,
}

impl From<iroh::net::endpoint::DirectAddrInfo> for DirectAddrInfo {
    fn from(value: iroh::net::endpoint::DirectAddrInfo) -> Self {
        Self {
            addr: value.addr.to_string(),
            latency: value.latency.map(|d| u32::try_from(d.as_millis()).unwrap()),
            last_control_time: value
                .last_control
                .as_ref()
                .map(|(d, _)| u32::try_from(d.as_millis()).unwrap()),
            last_control_msg: value.last_control.as_ref().map(|(_, m)| m.to_string()),
            last_payload: value
                .last_payload
                .map(|d| u32::try_from(d.as_millis()).unwrap()),
            last_alive: value
                .last_alive
                .map(|d| u32::try_from(d.as_millis()).unwrap()),
        }
    }
}

/// The latency and type of the control message
#[derive(Debug)]
#[napi(object)]
pub struct LatencyAndControlMsg {
    /// The latency of the control message. In milliseconds
    pub latency: u32,
    /// The type of control message, represented as a string
    pub control_msg: String,
}

/// Information about a connection
#[derive(Debug)]
#[napi(object)]
pub struct RemoteInfo {
    /// The node identifier of the endpoint. Also a public key.
    pub node_id: Vec<u8>,
    /// Relay url, if available.
    pub relay_url: Option<String>,
    /// List of addresses at which this node might be reachable, plus any latency information we
    /// have about that address and the last time the address was used.
    pub addrs: Vec<DirectAddrInfo>,
    /// The type of connection we have to the peer, either direct or over relay.
    pub conn_type: ConnectionType,
    /// The latency of the `conn_type`. In milliseconds.
    pub latency: Option<u32>,
    /// Duration since the last time this peer was used. In milliseconds.
    pub last_used: Option<u32>,
}

impl From<iroh::net::endpoint::RemoteInfo> for RemoteInfo {
    fn from(value: iroh::net::endpoint::RemoteInfo) -> Self {
        RemoteInfo {
            node_id: value.node_id.as_bytes().to_vec(),
            relay_url: value.relay_url.map(|info| info.relay_url.to_string()),
            addrs: value.addrs.into_iter().map(|a| a.into()).collect(),
            conn_type: value.conn_type.into(),
            latency: value.latency.map(|d| u32::try_from(d.as_micros()).unwrap()),
            last_used: value
                .last_used
                .map(|d| u32::try_from(d.as_micros()).unwrap()),
        }
    }
}

/// The type of the connection
#[derive(Debug)]
#[napi(string_enum)]
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
#[napi(object)]
pub struct ConnectionType {
    /// The type of connection.
    pub r#type: ConnType,
    /// Details of the actual connection, dependent on the type.
    pub details: Option<String>,
}

impl From<iroh::net::endpoint::ConnectionType> for ConnectionType {
    fn from(value: iroh::net::endpoint::ConnectionType) -> Self {
        match value {
            iroh::net::endpoint::ConnectionType::Direct(addr) => ConnectionType {
                r#type: ConnType::Direct,
                details: Some(addr.to_string()),
            },
            iroh::net::endpoint::ConnectionType::Mixed(addr, url) => ConnectionType {
                r#type: ConnType::Mixed,
                details: Some(format!("{} - {}", addr, url)),
            },
            iroh::net::endpoint::ConnectionType::Relay(url) => ConnectionType {
                r#type: ConnType::Relay,
                details: Some(url.to_string()),
            },
            iroh::net::endpoint::ConnectionType::None => ConnectionType {
                r#type: ConnType::None,
                details: None,
            },
        }
    }
}

/// A peer and it's addressing information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi(object)]
pub struct NodeAddr {
    pub node_id: String,
    /// Get the home relay URL for this peer
    pub relay_url: Option<String>,
    /// Direct addresses of this peer.
    pub addresses: Option<Vec<String>>,
}

/// Verify a `NodeAddr`.
#[napi]
pub fn verify_node_addr(addr: NodeAddr) -> Result<()> {
    let _addr: iroh::net::endpoint::NodeAddr = addr.try_into()?;
    Ok(())
}

impl TryFrom<NodeAddr> for iroh::net::endpoint::NodeAddr {
    type Error = anyhow::Error;

    fn try_from(value: NodeAddr) -> anyhow::Result<Self> {
        let key: iroh::net::key::PublicKey = value.node_id.parse().map_err(anyhow::Error::from)?;
        let mut node_addr = iroh::net::endpoint::NodeAddr::new(key);
        let addresses = value
            .addresses
            .unwrap_or_default()
            .into_iter()
            .map(|addr| {
                let addr: std::net::SocketAddr = addr.parse().map_err(anyhow::Error::from)?;
                Ok(addr)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        if let Some(derp_url) = value.relay_url {
            let url = url::Url::parse(&derp_url).map_err(anyhow::Error::from)?;

            node_addr = node_addr.with_relay_url(url.into());
        }
        node_addr = node_addr.with_direct_addresses(addresses);
        Ok(node_addr)
    }
}

impl From<iroh::net::endpoint::NodeAddr> for NodeAddr {
    fn from(value: iroh::net::endpoint::NodeAddr) -> Self {
        let addresses: Vec<_> = value
            .info
            .direct_addresses
            .into_iter()
            .map(|d| d.to_string())
            .collect();
        let addresses = if addresses.is_empty() {
            None
        } else {
            Some(addresses)
        };
        NodeAddr {
            node_id: value.node_id.to_string(),
            relay_url: value.info.relay_url.map(|url| url.to_string()),
            addresses,
        }
    }
}
