use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{
    blob::{BlobDownloadOptions, BlobFormat},
    doc::CapabilityKind,
    NodeAddr,
};

/// A token containing everything to get a file from the provider.
///
/// It is a single item which can be easily serialized and deserialized.
#[derive(Debug, PartialEq)]
#[napi]
pub struct BlobTicket {
    /// The provider to get a file from.
    #[napi(readonly)]
    pub node_addr: NodeAddr,
    /// The format of the blob.
    #[napi(readonly)]
    pub format: BlobFormat,
    /// The hash to retrieve.
    #[napi(readonly)]
    pub hash: String,
}

impl From<iroh::base::ticket::BlobTicket> for BlobTicket {
    fn from(value: iroh::base::ticket::BlobTicket) -> Self {
        Self {
            node_addr: value.node_addr().clone().into(),
            format: value.format().into(),
            hash: value.hash().to_string(),
        }
    }
}

impl TryFrom<&BlobTicket> for iroh::base::ticket::BlobTicket {
    type Error = anyhow::Error;

    fn try_from(value: &BlobTicket) -> anyhow::Result<Self> {
        let ticket = iroh::base::ticket::BlobTicket::new(
            value.node_addr.clone().try_into()?,
            value.hash.parse()?,
            value.format.into(),
        )?;
        Ok(ticket)
    }
}

#[napi]
impl BlobTicket {
    #[napi(constructor)]
    pub fn new(node_addr: NodeAddr, hash: String, format: BlobFormat) -> Self {
        Self {
            node_addr,
            hash,
            format,
        }
    }

    #[napi(factory)]
    pub fn from_string(str: String) -> Result<Self> {
        let ticket = iroh::base::ticket::BlobTicket::from_str(&str).map_err(anyhow::Error::from)?;
        Ok(ticket.into())
    }

    /// Checks if the two tickets are equal
    #[napi]
    pub fn is_equal(&self, other: &BlobTicket) -> bool {
        self == other
    }

    #[napi]
    pub fn to_string(&self) -> Result<String> {
        let ticket: iroh::base::ticket::BlobTicket = self.try_into()?;
        Ok(ticket.to_string())
    }

    /// True if the ticket is for a collection and should retrieve all blobs in it.
    #[napi]
    pub fn recursive(&self) -> bool {
        matches!(self.format, BlobFormat::HashSeq)
    }

    /// Convert this ticket into input parameters for a call to blobs_download
    #[napi]
    pub fn as_download_options(&self) -> Result<BlobDownloadOptions> {
        let res = iroh::client::blobs::DownloadOptions {
            format: self.format.into(),
            nodes: vec![self.node_addr.clone().try_into()?],
            tag: iroh::blobs::util::SetTagOption::Auto,
            mode: iroh::client::blobs::DownloadMode::Direct,
        }
        .into();
        Ok(res)
    }
}

/// Options when creating a ticket
#[derive(Debug)]
#[napi(string_enum)]
pub enum AddrInfoOptions {
    /// Only the Node ID is added.
    ///
    /// This usually means that iroh-dns discovery is used to find address information.
    Id,
    /// Include both the relay URL and the direct addresses.
    RelayAndAddresses,
    /// Only include the relay URL.
    Relay,
    /// Only include the direct addresses.
    Addresses,
}

impl From<AddrInfoOptions> for iroh::base::node_addr::AddrInfoOptions {
    fn from(options: AddrInfoOptions) -> iroh::base::node_addr::AddrInfoOptions {
        match options {
            AddrInfoOptions::Id => iroh::base::node_addr::AddrInfoOptions::Id,
            AddrInfoOptions::RelayAndAddresses => {
                iroh::base::node_addr::AddrInfoOptions::RelayAndAddresses
            }
            AddrInfoOptions::Relay => iroh::base::node_addr::AddrInfoOptions::Relay,
            AddrInfoOptions::Addresses => iroh::base::node_addr::AddrInfoOptions::Addresses,
        }
    }
}

/// Contains both a key (either secret or public) to a document, and a list of peers to join.
#[derive(Clone, Debug)]
#[napi]
pub struct DocTicket {
    /// The actual capability.
    #[napi(readonly)]
    pub capability: String,
    /// The capabillity kind
    #[napi(readonly)]
    pub capability_kind: CapabilityKind,
    /// A list of nodes to contact.
    #[napi(readonly)]
    pub nodes: Vec<NodeAddr>,
}

impl From<iroh::docs::DocTicket> for DocTicket {
    fn from(value: iroh::docs::DocTicket) -> Self {
        let (capability, kind) = match value.capability {
            iroh::docs::Capability::Read(v) => (v.to_string(), CapabilityKind::Read),
            iroh::docs::Capability::Write(v) => (v.to_string(), CapabilityKind::Write),
        };
        Self {
            capability,
            capability_kind: kind,
            nodes: value.nodes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<&DocTicket> for iroh::docs::DocTicket {
    type Error = anyhow::Error;

    fn try_from(value: &DocTicket) -> anyhow::Result<Self> {
        let peers = value
            .nodes
            .iter()
            .map(|v| v.clone().try_into())
            .collect::<anyhow::Result<_>>()?;

        let capability = match value.capability_kind {
            CapabilityKind::Read => iroh::docs::Capability::Read(value.capability.parse()?),
            CapabilityKind::Write => iroh::docs::Capability::Write(value.capability.parse()?),
        };

        let ticket = iroh::docs::DocTicket::new(capability, peers);
        Ok(ticket)
    }
}

#[napi]
impl DocTicket {
    #[napi(factory)]
    pub fn from_string(str: String) -> Result<Self> {
        let ticket = iroh::docs::DocTicket::from_str(&str).map_err(anyhow::Error::from)?;
        Ok(ticket.into())
    }

    #[napi]
    pub fn to_string(&self) -> Result<String> {
        let ticket: iroh::docs::DocTicket = self.try_into()?;
        Ok(ticket.to_string())
    }
}
