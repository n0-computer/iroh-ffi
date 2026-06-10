use std::{collections::BTreeSet, net::SocketAddr, str::FromStr, sync::Arc};

use iroh_base::{RelayUrl, TransportAddr};

use crate::{EndpointId, IrohError};

/// An endpoint's id together with the network-level addresses where it can be reached.
///
/// Mirrors `iroh::EndpointAddr` — exposes a flat view over the underlying set of
/// `TransportAddr`s (one relay URL plus a list of IP/port pairs).
#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Object)]
#[uniffi::export(Display, Eq, Hash)]
pub struct EndpointAddr {
    pub(crate) id: Arc<EndpointId>,
    pub(crate) relay_url: Option<String>,
    pub(crate) addresses: Vec<String>,
}

impl std::fmt::Display for EndpointAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)?;
        if let Some(relay) = &self.relay_url {
            write!(f, " relay={relay}")?;
        }
        if !self.addresses.is_empty() {
            write!(f, " addrs=[{}]", self.addresses.join(", "))?;
        }
        Ok(())
    }
}

#[uniffi::export]
impl EndpointAddr {
    /// Create a new [`EndpointAddr`].
    #[uniffi::constructor]
    pub fn new(id: &EndpointId, relay_url: Option<String>, addresses: Vec<String>) -> Self {
        Self {
            id: Arc::new(id.clone()),
            relay_url,
            addresses,
        }
    }

    /// The endpoint id.
    pub fn id(&self) -> Arc<EndpointId> {
        self.id.clone()
    }

    /// The direct (IP/port) addresses of this peer.
    pub fn direct_addresses(&self) -> Vec<String> {
        self.addresses.clone()
    }

    /// The home relay URL for this peer, if known.
    pub fn relay_url(&self) -> Option<String> {
        self.relay_url.clone()
    }
}

impl TryFrom<EndpointAddr> for iroh::EndpointAddr {
    type Error = IrohError;

    fn try_from(value: EndpointAddr) -> Result<Self, Self::Error> {
        let mut addrs: BTreeSet<TransportAddr> = BTreeSet::new();
        for addr in &value.addresses {
            let socket = SocketAddr::from_str(addr).map_err(anyhow::Error::from)?;
            addrs.insert(TransportAddr::Ip(socket));
        }
        if let Some(relay_url) = &value.relay_url {
            let url = RelayUrl::from_str(relay_url).map_err(anyhow::Error::from)?;
            addrs.insert(TransportAddr::Relay(url));
        }
        Ok(iroh::EndpointAddr {
            id: (&*value.id).into(),
            addrs,
        })
    }
}

impl From<iroh::EndpointAddr> for EndpointAddr {
    fn from(value: iroh::EndpointAddr) -> Self {
        let mut relay_url = None;
        let mut addresses = Vec::new();
        for addr in &value.addrs {
            match addr {
                TransportAddr::Relay(url) => {
                    relay_url = Some(url.to_string());
                }
                TransportAddr::Ip(socket) => {
                    addresses.push(socket.to_string());
                }
                _ => {}
            }
        }
        EndpointAddr {
            id: Arc::new(value.id.into()),
            relay_url,
            addresses,
        }
    }
}
