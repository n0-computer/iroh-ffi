use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{
    blob::{BlobDownloadOptions, BlobFormat, Hash},
    NodeAddr,
};

/// A token containing everything to get a file from the provider.
///
/// It is a single item which can be easily serialized and deserialized.
#[derive(Debug)]
#[napi]
pub struct BlobTicket(iroh::base::ticket::BlobTicket);

#[napi]
impl BlobTicket {
    #[napi(constructor)]
    pub fn new(str: String) -> Result<Self> {
        let ticket = iroh::base::ticket::BlobTicket::from_str(&str).map_err(anyhow::Error::from)?;
        Ok(BlobTicket(ticket))
    }

    /// The hash of the item this ticket can retrieve.
    #[napi]
    pub fn hash(&self) -> Hash {
        self.0.hash().into()
    }

    /// The [`NodeAddr`] of the provider for this ticket.
    #[napi]
    pub fn node_addr(&self) -> NodeAddr {
        let addr = self.0.node_addr().clone();
        addr.into()
    }

    /// The [`BlobFormat`] for this ticket.
    #[napi]
    pub fn format(&self) -> BlobFormat {
        self.0.format().into()
    }

    /// True if the ticket is for a collection and should retrieve all blobs in it.
    #[napi]
    pub fn recursive(&self) -> bool {
        self.0.format().is_hash_seq()
    }

    /// Convert this ticket into input parameters for a call to blobs_download
    #[napi]
    pub fn as_download_options(&self) -> BlobDownloadOptions {
        iroh::client::blobs::DownloadOptions {
            format: self.0.format(),
            nodes: vec![self.0.node_addr().clone()],
            tag: iroh::blobs::util::SetTagOption::Auto,
            mode: iroh::client::blobs::DownloadMode::Direct,
        }
        .into()
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
