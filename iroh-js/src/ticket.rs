use iroh_tickets::Ticket as _;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::EndpointAddr;

/// A token containing information for establishing a connection to an endpoint.
#[derive(Debug, Clone)]
#[napi]
pub struct EndpointTicket(iroh_tickets::endpoint::EndpointTicket);

impl From<iroh_tickets::endpoint::EndpointTicket> for EndpointTicket {
    fn from(t: iroh_tickets::endpoint::EndpointTicket) -> Self {
        EndpointTicket(t)
    }
}

#[napi]
impl EndpointTicket {
    /// Wrap an [`EndpointAddr`] into a ticket.
    #[napi(factory)]
    pub fn from_addr(addr: &EndpointAddr) -> Result<Self> {
        let inner: iroh::EndpointAddr = addr.try_into()?;
        Ok(iroh_tickets::endpoint::EndpointTicket::new(inner).into())
    }

    /// Parse a ticket from its base32 string form.
    #[napi(factory)]
    pub fn from_string(s: String) -> Result<Self> {
        let ticket = iroh_tickets::endpoint::EndpointTicket::decode_string(&s)
            .map_err(anyhow::Error::from)?;
        Ok(EndpointTicket(ticket))
    }

    /// The [`EndpointAddr`] embedded in this ticket.
    #[napi]
    pub fn endpoint_addr(&self) -> EndpointAddr {
        self.0.endpoint_addr().clone().into()
    }

    /// Base32 string form.
    #[napi]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
