use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use futures::stream::TryStreamExt;
use iroh::{
    bytes::util::runtime::Handle,
    net::key::SecretKey,
    node::{Node, DEFAULT_BIND_ADDR},
    rpc_protocol::{ProviderRequest, ProviderResponse},
};
use quic_rpc::transport::flume::FlumeConnection;

use crate::block_on;
use crate::doc::{AuthorId, CapabilityKind, Doc, DocTicket, NamespaceId};
use crate::error::IrohError as Error;
use crate::key::PublicKey;

pub use iroh::rpc_protocol::CounterStats;

/// Information about a direct address.
/// TODO: when refactoring the iroh.node API, give this an impl
#[derive(Debug)]
pub struct DirectAddrInfo(iroh::net::magicsock::DirectAddrInfo);

#[derive(Debug)]
pub struct ConnectionInfo {
    /// The public key of the endpoint.
    pub public_key: Arc<PublicKey>,
    /// Derp region, if available.
    pub derp_region: Option<u16>,
    /// List of addresses at which this node might be reachable, plus any latency information we
    /// have about that address and the last time the address was used.
    pub addrs: Vec<Arc<DirectAddrInfo>>,
    /// The type of connection we have to the peer, either direct or over relay.
    pub conn_type: ConnectionType,
    /// The latency of the `conn_type`.
    pub latency: Option<Duration>,
    /// Duration since the last time this peer was used.
    pub last_used: Option<Duration>,
}

impl From<iroh::net::magic_endpoint::ConnectionInfo> for ConnectionInfo {
    fn from(value: iroh::net::magic_endpoint::ConnectionInfo) -> Self {
        ConnectionInfo {
            public_key: Arc::new(PublicKey(value.public_key)),
            derp_region: value.derp_region,
            addrs: value
                .addrs
                .iter()
                .map(|a| Arc::new(DirectAddrInfo(a.clone())))
                .collect(),
            conn_type: value.conn_type.into(),
            latency: value.latency,
            last_used: value.last_used,
        }
    }
}

#[derive(Debug)]
pub enum ConnectionType {
    Direct { addr: String, port: u16 },
    Relay { port: u16 },
    Mixed { addr: String, port: u16 },
    None,
}

impl From<iroh::net::magicsock::ConnectionType> for ConnectionType {
    fn from(value: iroh::net::magicsock::ConnectionType) -> Self {
        match value {
            iroh::net::magicsock::ConnectionType::Direct(addr) => ConnectionType::Direct {
                addr: addr.ip().to_string(),
                port: addr.port(),
            },
            iroh::net::magicsock::ConnectionType::Mixed(addr, port) => ConnectionType::Mixed {
                addr: addr.ip().to_string(),
                port,
            },
            iroh::net::magicsock::ConnectionType::Relay(port) => ConnectionType::Relay { port },
            iroh::net::magicsock::ConnectionType::None => ConnectionType::None,
        }
    }
}

pub struct IrohNode {
    pub(crate) node: Node<iroh::bytes::store::flat::Store>,
    pub(crate) async_runtime: Handle,
    pub(crate) sync_client: iroh::client::Iroh<FlumeConnection<ProviderResponse, ProviderRequest>>,
    #[allow(dead_code)]
    pub(crate) tokio_rt: tokio::runtime::Runtime,
}

impl IrohNode {
    pub fn new(path: String) -> Result<Self, Error> {
        let path = PathBuf::from(path);
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(Error::runtime)?;

        let tpc = tokio_util::task::LocalPoolHandle::new(num_cpus::get());
        let rt = iroh::bytes::util::runtime::Handle::new(tokio_rt.handle().clone(), tpc);

        let rt_inner = rt.clone();
        let node = block_on(&rt, async move {
            // TODO: store and load keypair
            let secret_key = SecretKey::generate();

            let docs_path = path.join("docs.db");
            let docs = iroh::sync::store::fs::Store::new(&docs_path)?;

            // create a bao store for the iroh-bytes blobs
            let blob_path = path.join("blobs");
            tokio::fs::create_dir_all(&blob_path).await?;
            let db = iroh::bytes::store::flat::Store::load(
                &blob_path, &blob_path, &blob_path, &rt_inner,
            )
            .await?;

            Node::builder(db, docs)
                .bind_addr(DEFAULT_BIND_ADDR.into())
                .secret_key(secret_key)
                .runtime(&rt_inner)
                .spawn()
                .await
        })
        .map_err(Error::node_create)?;

        let sync_client = node.client();

        Ok(IrohNode {
            node,
            async_runtime: rt,
            sync_client,
            tokio_rt,
        })
    }

    pub fn node_id(&self) -> String {
        self.node.node_id().to_string()
    }

    pub fn doc_create(&self) -> Result<Arc<Doc>, Error> {
        block_on(&self.async_runtime, async {
            let doc = self.sync_client.docs.create().await.map_err(Error::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.async_runtime.clone(),
            }))
        })
    }

    pub fn author_create(&self) -> Result<Arc<AuthorId>, Error> {
        block_on(&self.async_runtime, async {
            let author = self
                .sync_client
                .authors
                .create()
                .await
                .map_err(Error::author)?;

            Ok(Arc::new(AuthorId(author)))
        })
    }

    pub fn author_list(&self) -> Result<Vec<Arc<AuthorId>>, Error> {
        block_on(&self.async_runtime, async {
            let authors = self
                .sync_client
                .authors
                .list()
                .await
                .map_err(Error::author)?
                .map_ok(|id| Arc::new(AuthorId(id)))
                .try_collect::<Vec<_>>()
                .await
                .map_err(Error::author)?;
            Ok(authors)
        })
    }

    pub fn doc_join(&self, ticket: Arc<DocTicket>) -> Result<Arc<Doc>, Error> {
        block_on(&self.async_runtime, async {
            let doc = self
                .sync_client
                .docs
                .import(ticket.0.clone())
                .await
                .map_err(Error::doc)?;

            Ok(Arc::new(Doc {
                inner: doc,
                rt: self.async_runtime.clone(),
            }))
        })
    }

    pub fn doc_list(&self) -> Result<Vec<NamespaceAndCapability>, Error> {
        block_on(&self.async_runtime, async {
            let docs = self
                .sync_client
                .docs
                .list()
                .await
                .map_err(Error::doc)?
                .map_ok(|(namespace, capability)| NamespaceAndCapability {
                    namespace: Arc::new(namespace.into()),
                    capability,
                })
                .try_collect::<Vec<_>>()
                .await
                .map_err(Error::doc)?;

            Ok(docs)
        })
    }

    pub fn stats(&self) -> Result<HashMap<String, CounterStats>, Error> {
        block_on(&self.async_runtime, async {
            let stats = self.sync_client.node.stats().await.map_err(Error::doc)?;
            Ok(stats)
        })
    }

    pub fn connections(&self) -> Result<Vec<ConnectionInfo>, Error> {
        block_on(&self.async_runtime, async {
            let infos = self
                .sync_client
                .node
                .connections()
                .await
                .map_err(Error::connection)?
                .map_ok(|info| info.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(Error::connection)?;
            Ok(infos)
        })
    }

    pub fn connection_info(
        &self,
        node_id: Arc<PublicKey>,
    ) -> Result<Option<ConnectionInfo>, Error> {
        block_on(&self.async_runtime, async {
            let info = self
                .sync_client
                .node
                .connection_info(node_id.as_ref().0)
                .await
                .map(|i| i.map(|i| i.into()))
                .map_err(Error::connection)?;
            Ok(info)
        })
    }
}

pub struct NamespaceAndCapability {
    pub namespace: Arc<NamespaceId>,
    pub capability: CapabilityKind,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{Hash, IrohError, LiveEvent, ShareMode, SubscribeCallback};

    #[test]
    fn test_doc_create() {
        let path = tempfile::tempdir().unwrap();
        let node = IrohNode::new(path.path().to_string_lossy().into_owned()).unwrap();
        let node_id = node.node_id();
        println!("id: {}", node_id);
        let doc = node.doc_create().unwrap();
        let doc_id = doc.id();
        println!("doc_id: {}", doc_id);

        let doc_ticket = doc.share(crate::doc::ShareMode::Write).unwrap();
        let doc_ticket_string = doc_ticket.to_string();
        let dock_ticket_back = DocTicket::from_string(doc_ticket_string.clone()).unwrap();
        assert_eq!(doc_ticket.0.to_string(), dock_ticket_back.0.to_string());
        println!("doc_ticket: {}", doc_ticket_string);
        node.doc_join(doc_ticket).unwrap();
    }

    #[test]
    fn test_basic_sync() {
        // create node_0
        let iroh_dir_0 = tempfile::tempdir().unwrap();
        let node_0 = IrohNode::new(iroh_dir_0.path().to_string_lossy().into_owned()).unwrap();

        // create node_1
        let iroh_dir_1 = tempfile::tempdir().unwrap();
        let node_1 = IrohNode::new(iroh_dir_1.path().to_string_lossy().into_owned()).unwrap();

        // create doc on node_0
        let doc_0 = node_0.doc_create().unwrap();
        let ticket = doc_0.share(ShareMode::Write).unwrap();

        // subscribe to sync events
        let (found_s, found_r) = std::sync::mpsc::channel();
        struct Callback {
            found_s: std::sync::mpsc::Sender<Result<Hash, IrohError>>,
        }
        impl SubscribeCallback for Callback {
            fn event(&self, event: Arc<LiveEvent>) -> Result<(), IrohError> {
                match *event {
                    LiveEvent::ContentReady { ref hash } => {
                        self.found_s
                            .send(Ok(hash.clone()))
                            .map_err(IrohError::doc)?;
                    }
                    _ => {}
                }
                Ok(())
            }
        }
        let cb = Callback { found_s };
        doc_0.subscribe(Box::new(cb)).unwrap();

        // join the same doc from node_1
        let doc_1 = node_1.doc_join(ticket).unwrap();

        // create author on node_1
        let author = node_1.author_create().unwrap();
        doc_1
            .set_bytes(author, b"hello".to_vec(), b"world".to_vec())
            .unwrap();
        let hash = found_r.recv().unwrap().unwrap();
        let val = node_1.blobs_read_to_bytes(hash.into()).unwrap();
        assert_eq!(b"world".to_vec(), val);
    }
}
