use crate::{Iroh, IrohError, NodeAddr, PublicKey};
use iroh::Endpoint;

/// Iroh net client.
#[derive(uniffi::Object)]
pub struct Net {
    endpoint: Endpoint,
}

#[uniffi::export]
impl Iroh {
    /// Access to blob specific funtionaliy.
    pub fn net(&self) -> Net {
        let endpoint = self.raw_endpoint().clone();
        Net { endpoint }
    }
}

#[uniffi::export]
impl Net {
    /// The string representation of the PublicKey of this node.
    pub async fn node_id(&self) -> Result<String, IrohError> {
        let id = self.endpoint.node_id().await?;
        Ok(id.to_string())
    }

    /// Return the [`NodeAddr`] for this node.
    pub async fn node_addr(&self) -> Result<NodeAddr, IrohError> {
        let addr = self.endpoint.node_addr().await?;
        Ok(addr.into())
    }

    /// Get the relay server we are connected to.
    pub async fn home_relay(&self) -> Result<Option<String>, IrohError> {
        let relay = self.endpoint.home_relay().await?;
        Ok(relay.map(|u| u.to_string()))
    }

    /// Returns the latency to the given node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn latency(&self, node_id: &PublicKey) -> Result<Option<u64>, IrohError> {
        let info = self
            .endpoint
            .latency(node_id.into())
            .await
            .map(|i| i.as_millis().try_into().expect("duration too large"))?;
        Ok(info)
    }
}
