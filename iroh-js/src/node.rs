use std::{collections::HashMap, path::PathBuf, time::Duration};

use iroh::node::{FsNode, MemNode};
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;

use crate::{BlobProvideEvent, CounterStats, NodeAddr};

/// Options passed to [`IrohNode.new`]. Controls the behaviour of an iroh node.#
#[napi(object, object_to_js = false)]
pub struct NodeOptions {
    /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
    /// Set to null to disable gc
    pub gc_interval_millis: Option<u32>,
    /// Provide a callback to hook into events when the blobs component adds and provides blobs.
    pub blob_events: Option<ThreadsafeFunction<BlobProvideEvent, ()>>,
}

impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            gc_interval_millis: None,
            blob_events: None,
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
        if let Some(blob_events_cb) = options.blob_events {
            builder = builder.blobs_events(BlobProvideEvents::new(blob_events_cb))
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

        if let Some(blob_events_cb) = options.blob_events {
            builder = builder.blobs_events(BlobProvideEvents::new(blob_events_cb))
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

    /// Get status information about a node
    #[napi]
    pub async fn status(&self) -> Result<NodeStatus> {
        let res = self.node().status().await.map(|n| n.into())?;
        Ok(res)
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

#[derive(Clone)]
struct BlobProvideEvents {
    callback: ThreadsafeFunction<BlobProvideEvent, ()>,
}

impl std::fmt::Debug for BlobProvideEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobProvideEvents()")
    }
}

impl BlobProvideEvents {
    fn new(callback: ThreadsafeFunction<BlobProvideEvent, ()>) -> Self {
        Self { callback }
    }
}

impl iroh::blobs::provider::CustomEventSender for BlobProvideEvents {
    fn send(&self, event: iroh::blobs::provider::Event) -> futures::future::BoxFuture<'static, ()> {
        let cb = self.callback.clone();
        Box::pin(async move {
            let msg = BlobProvideEvent::convert(event);
            if let Err(err) = cb.call_async(msg).await {
                eprintln!("failed call: {:?}", err);
            }
        })
    }

    fn try_send(&self, event: iroh::blobs::provider::Event) {
        let cb = self.callback.clone();
        let msg = BlobProvideEvent::convert(event);
        cb.call(msg, ThreadsafeFunctionCallMode::NonBlocking);
    }
}
