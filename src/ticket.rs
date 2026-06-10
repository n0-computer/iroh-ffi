use std::sync::Arc;

use iroh_tickets::Ticket as _;

use crate::{EndpointAddr, IrohError};

/// A token containing information for establishing a connection to an endpoint.
///
/// This allows establishing a connection to the endpoint in most circumstances where
/// it is possible to do so. It is a single item that can be easily serialized and
/// deserialized to/from a base32 string.
#[derive(Debug, uniffi::Object)]
#[uniffi::export(Display)]
pub struct EndpointTicket(iroh_tickets::endpoint::EndpointTicket);

impl From<iroh_tickets::endpoint::EndpointTicket> for EndpointTicket {
    fn from(ticket: iroh_tickets::endpoint::EndpointTicket) -> Self {
        EndpointTicket(ticket)
    }
}

impl std::fmt::Display for EndpointTicket {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[uniffi::export]
impl EndpointTicket {
    /// Wrap the given [`EndpointAddr`] as an [`EndpointTicket`].
    ///
    /// The returned ticket can be serialized via [`Self::to_string`] and parsed back
    /// using [`Self::from_string`].
    #[uniffi::constructor]
    pub fn from_addr(addr: &EndpointAddr) -> Result<Self, IrohError> {
        let inner: iroh::EndpointAddr = addr.clone().try_into()?;
        Ok(iroh_tickets::endpoint::EndpointTicket::new(inner).into())
    }

    /// Parse an [`EndpointTicket`] from its string presentation.
    #[uniffi::constructor]
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let ticket = iroh_tickets::endpoint::EndpointTicket::decode_string(&str)?;
        Ok(EndpointTicket(ticket))
    }

    /// The [`EndpointAddr`] embedded in this ticket.
    pub fn endpoint_addr(&self) -> Arc<EndpointAddr> {
        let addr = self.0.endpoint_addr().clone();
        Arc::new(addr.into())
    }
}
