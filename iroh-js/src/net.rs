use std::{collections::BTreeSet, net::SocketAddr, str::FromStr};

use iroh_base::{RelayUrl, TransportAddr};
use napi_derive::napi;

use crate::EndpointId;

/// An endpoint's id together with the network-level addresses where it can be
/// reached. Mirrors the uniffi `EndpointAddr` object surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct EndpointAddr {
    pub(crate) id: [u8; 32],
    pub(crate) relay_url: Option<String>,
    pub(crate) addresses: Vec<String>,
}

#[napi]
impl EndpointAddr {
    /// Create a new [`EndpointAddr`].
    #[napi(constructor)]
    pub fn new(id: &EndpointId, relay_url: Option<String>, addresses: Option<Vec<String>>) -> Self {
        Self {
            id: id.raw_bytes(),
            relay_url,
            addresses: addresses.unwrap_or_default(),
        }
    }

    /// The endpoint id.
    #[napi]
    pub fn id(&self) -> EndpointId {
        EndpointId::from_raw_bytes(self.id)
    }

    /// The direct (ip:port) addresses of this peer.
    #[napi]
    pub fn direct_addresses(&self) -> Vec<String> {
        self.addresses.clone()
    }

    /// The home relay URL for this peer, if known.
    #[napi]
    pub fn relay_url(&self) -> Option<String> {
        self.relay_url.clone()
    }

    /// Returns true if both [`EndpointAddr`]s have the same values.
    #[napi]
    pub fn equal(&self, other: &EndpointAddr) -> bool {
        self == other
    }

    /// Clean string representation.
    #[napi]
    pub fn to_string(&self) -> String {
        let id = iroh::EndpointId::from_bytes(&self.id)
            .map(|i| i.to_string())
            .unwrap_or_else(|_| "<invalid>".to_string());
        let mut s = id;
        if let Some(relay) = &self.relay_url {
            s.push_str(&format!(" relay={relay}"));
        }
        if !self.addresses.is_empty() {
            s.push_str(&format!(" addrs=[{}]", self.addresses.join(", ")));
        }
        s
    }
}

impl TryFrom<&EndpointAddr> for iroh::EndpointAddr {
    type Error = anyhow::Error;

    fn try_from(value: &EndpointAddr) -> anyhow::Result<Self> {
        let id = iroh::EndpointId::from_bytes(&value.id)?;
        let mut addrs: BTreeSet<TransportAddr> = BTreeSet::new();
        for addr in &value.addresses {
            let socket = SocketAddr::from_str(addr)?;
            addrs.insert(TransportAddr::Ip(socket));
        }
        if let Some(relay) = &value.relay_url {
            let url = RelayUrl::from_str(relay)?;
            addrs.insert(TransportAddr::Relay(url));
        }
        Ok(iroh::EndpointAddr { id, addrs })
    }
}

impl From<iroh::EndpointAddr> for EndpointAddr {
    fn from(value: iroh::EndpointAddr) -> Self {
        let mut relay_url = None;
        let mut addresses = Vec::new();
        for addr in &value.addrs {
            match addr {
                TransportAddr::Relay(url) => relay_url = Some(url.to_string()),
                TransportAddr::Ip(socket) => addresses.push(socket.to_string()),
                _ => {}
            }
        }
        EndpointAddr {
            id: *value.id.as_bytes(),
            relay_url,
            addresses,
        }
    }
}
