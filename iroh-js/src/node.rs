use std::{collections::HashMap, path::PathBuf, time::Duration};

use futures::stream::TryStreamExt;
use iroh::node::{FsNode, MemNode};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{ConnectionInfo, CounterStats, NodeAddr, PublicKey};

/// Options passed to [`IrohNode.new`]. Controls the behaviour of an iroh node.
#[derive(Debug)]
#[napi(object)]
pub struct NodeOptions {
    /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
    /// Set to null to disable gc
    pub gc_interval_millis: Option<u32>,
}

impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            gc_interval_millis: None,
        }
    }
}

/// An Iroh node. Allows you to sync, store, and transfer data.
#[derive(Debug, Clone)]
#[napi]
pub struct Iroh(InnerIroh);

#[derive(Debug, Clone)]
enum InnerIroh {
    Fs(FsNode),
    Memory(MemNode),
}

impl Iroh {
    pub(crate) fn client(&self) -> &iroh::client::Iroh {
        match &self.0 {
            InnerIroh::Fs(node) => node,
            InnerIroh::Memory(node) => node,
        }
    }
}

#[napi]
impl Iroh {
    /// Create a new iroh node.
    ///
    /// The `path` param should be a directory where we can store or load
    /// iroh data from a previous session.
    #[napi(factory)]
    pub async fn persistent(path: String, opts: Option<NodeOptions>) -> Result<Self> {
        let options = opts.unwrap_or_default();

        let path = PathBuf::from(path);

        let mut builder = iroh::node::Builder::default().persist(path).await?;
        if let Some(millis) = options.gc_interval_millis {
            let policy = match millis {
                0 => iroh::node::GcPolicy::Disabled,
                millis => iroh::node::GcPolicy::Interval(Duration::from_millis(millis as _)),
            };
            builder = builder.gc_policy(policy);
        }
        let node = builder.spawn().await?;

        Ok(Iroh(InnerIroh::Fs(node)))
    }

    /// Create a new iroh node.
    ///
    /// All data will be only persistet in memory.
    #[napi(factory)]
    pub async fn memory(opts: Option<NodeOptions>) -> Result<Self> {
        let options = opts.unwrap_or_default();

        let mut builder = iroh::node::Builder::default();
        if let Some(millis) = options.gc_interval_millis {
            let policy = match millis {
                0 => iroh::node::GcPolicy::Disabled,
                millis => iroh::node::GcPolicy::Interval(Duration::from_millis(millis as _)),
            };
            builder = builder.gc_policy(policy);
        }
        let node = builder.spawn().await?;

        Ok(Iroh(InnerIroh::Memory(node)))
    }

    /// Access to node specific funtionaliy.
    #[napi(getter)]
    pub fn node(&self) -> Node {
        Node { node: self.clone() }
    }
}

/// Iroh node client.
#[napi]
pub struct Node {
    node: Iroh,
}

impl Node {
    fn node(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[napi]
impl Node {
    /// Get statistics of the running node.
    #[napi]
    pub async fn stats(&self) -> Result<HashMap<String, CounterStats>> {
        let stats = self.node().stats().await?;
        let stats = stats
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    CounterStats {
                        value: u32::try_from(v.value).expect("value too large"),
                        description: v.description,
                    },
                )
            })
            .collect();
        Ok(stats)
    }

    /// Return `ConnectionInfo`s for each connection we have to another iroh node.
    #[napi]
    pub async fn connections(&self) -> Result<Vec<ConnectionInfo>> {
        let infos = self
            .node()
            .remote_info_iter()
            .await?
            .map_ok(|info| info.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(infos)
    }

    /// Return connection information on the currently running node.
    #[napi]
    pub async fn connection_info(&self, node_id: &PublicKey) -> Result<Option<ConnectionInfo>> {
        let info = self
            .node()
            .remote_info(node_id.into())
            .await
            .map(|i| i.map(|i| i.into()))?;
        Ok(info)
    }

    /// Get status information about a node
    #[napi]
    pub async fn status(&self) -> Result<NodeStatus> {
        let res = self.node().status().await.map(|n| n.into())?;
        Ok(res)
    }

    /// The string representation of the PublicKey of this node.
    #[napi]
    pub async fn node_id(&self) -> Result<String> {
        let id = self.node().node_id().await?;
        Ok(id.to_string())
    }

    /// Return the [`NodeAddr`] for this node.
    #[napi]
    pub async fn node_addr(&self) -> Result<NodeAddr> {
        let addr = self.node().node_addr().await?;
        Ok(addr.into())
    }

    /// Add a known node address to the node.
    #[napi]
    pub async fn add_node_addr(&self, addr: NodeAddr) -> Result<()> {
        self.node().add_node_addr(addr.clone().try_into()?).await?;
        Ok(())
    }

    /// Get the relay server we are connected to.
    #[napi]
    pub async fn home_relay(&self) -> Result<Option<String>> {
        let relay = self.node().home_relay().await?;
        Ok(relay.map(|u| u.to_string()))
    }

    /// Shutdown this iroh node.
    #[napi]
    pub async fn shutdown(&self, force: bool) -> Result<()> {
        self.node().shutdown(force).await?;
        Ok(())
    }

    /// Returns `Some(addr)` if an RPC endpoint is running, `None` otherwise.
    #[napi]
    pub fn my_rpc_addr(&self) -> Option<String> {
        let addr = match self.node.0 {
            InnerIroh::Fs(ref n) => n.my_rpc_addr(),
            InnerIroh::Memory(ref n) => n.my_rpc_addr(),
        };
        addr.map(|a| a.to_string())
    }
}

/// The response to a status request
#[derive(Debug)]
#[napi(object)]
pub struct NodeStatus {
    /// The node id and socket addresses of this node.
    pub addr: NodeAddr,
    /// The bound listening addresses of the node
    pub listen_addrs: Vec<String>,
    /// The version of the node
    pub version: String,
    /// RPC address, if currently listening.
    pub rpc_addr: Option<String>,
}

impl From<iroh::client::NodeStatus> for NodeStatus {
    fn from(n: iroh::client::NodeStatus) -> Self {
        NodeStatus {
            addr: n.addr.into(),
            listen_addrs: n.listen_addrs.iter().map(|addr| addr.to_string()).collect(),
            version: n.version,
            rpc_addr: n.rpc_addr.map(|a| a.to_string()),
        }
    }
}
