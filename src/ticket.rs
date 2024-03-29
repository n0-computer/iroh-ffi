use std::str::FromStr;
use std::sync::Arc;

use crate::blob::{BlobDownloadRequest, BlobFormat, Hash};
use crate::doc::NodeAddr;
use crate::error::IrohError;

/// A token containing everything to get a file from the provider.
///
/// It is a single item which can be easily serialized and deserialized.
pub struct BlobTicket(iroh::base::ticket::BlobTicket);
impl BlobTicket {
    pub fn new(str: String) -> Result<Self, IrohError> {
        let ticket =
            iroh::base::ticket::BlobTicket::from_str(&str).map_err(IrohError::blob_ticket)?;
        Ok(BlobTicket(ticket))
    }

    /// The hash of the item this ticket can retrieve.
    pub fn hash(&self) -> Arc<Hash> {
        Arc::new(self.0.hash().into())
    }

    /// The [`NodeAddr`] of the provider for this ticket.
    pub fn node_addr(&self) -> Arc<NodeAddr> {
        let addr = self.0.node_addr().clone();
        Arc::new(addr.into())
    }

    /// The [`BlobFormat`] for this ticket.
    pub fn format(&self) -> BlobFormat {
        self.0.format().into()
    }

    /// True if the ticket is for a collection and should retrieve all blobs in it.
    pub fn recursive(&self) -> bool {
        self.0.format().is_hash_seq()
    }

    /// Convert this ticket into input parameters for a call to blobs_download
    pub fn as_download_request(&self) -> Arc<BlobDownloadRequest> {
        let r: BlobDownloadRequest = iroh::rpc_protocol::BlobDownloadRequest {
            hash: self.0.hash(),
            format: self.0.format(),
            peer: self.0.node_addr().clone(),
            tag: iroh::rpc_protocol::SetTagOption::Auto,
        }
        .into();
        Arc::new(r)
    }
}

/// Options when creating a ticket
pub enum ShareTicketOptions {
    /// Include both the relay URL and the direct addresses.
    RelayAndAddresses,
    /// Only include the relay URL.
    Relay,
    /// Only include the direct addresses.
    Addresses,
}

impl From<ShareTicketOptions> for iroh::client::ShareTicketOptions {
    fn from(options: ShareTicketOptions) -> iroh::client::ShareTicketOptions {
        match options {
            ShareTicketOptions::RelayAndAddresses => {
                iroh::client::ShareTicketOptions::RelayAndAddresses
            }
            ShareTicketOptions::Relay => iroh::client::ShareTicketOptions::Relay,
            ShareTicketOptions::Addresses => iroh::client::ShareTicketOptions::Addresses,
        }
    }
}
