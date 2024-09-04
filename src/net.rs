use futures::TryStreamExt;

use crate::{Iroh, IrohError, NodeAddr, PublicKey, RemoteInfo};

/// Iroh net client.
#[derive(uniffi::Object)]
pub struct Net {
    node: Iroh,
}

#[uniffi::export]
impl Iroh {
    /// Access to blob specific funtionaliy.
    pub fn net(&self) -> Net {
        Net { node: self.clone() }
    }
}

impl Net {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.inner_client()
    }
}

#[uniffi::export]
impl Net {
    /// The string representation of the PublicKey of this node.
    pub async fn node_id(&self) -> Result<String, IrohError> {
        let id = self.client().net().node_id().await?;
        Ok(id.to_string())
    }

    /// Return the [`NodeAddr`] for this node.
    pub async fn node_addr(&self) -> Result<NodeAddr, IrohError> {
        let addr = self.client().net().node_addr().await?;
        Ok(addr.into())
    }

    /// Add a known node address to the node.
    pub async fn add_node_addr(&self, addr: &NodeAddr) -> Result<(), IrohError> {
        self.client()
            .net()
            .add_node_addr(addr.clone().try_into()?)
            .await?;
        Ok(())
    }

    /// Get the relay server we are connected to.
    pub async fn home_relay(&self) -> Result<Option<String>, IrohError> {
        let relay = self.client().net().home_relay().await?;
        Ok(relay.map(|u| u.to_string()))
    }

    /// Return `ConnectionInfo`s for each connection we have to another iroh node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_info_list(&self) -> Result<Vec<RemoteInfo>, IrohError> {
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

    /// Return connection information on the currently running node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_info(&self, node_id: &PublicKey) -> Result<Option<RemoteInfo>, IrohError> {
        let info = self
            .client()
            .net()
            .remote_info(node_id.into())
            .await
            .map(|i| i.map(|i| i.into()))?;
        Ok(info)
    }
}
