use iroh::{
    baomap::mem,
    bytes::util::runtime::Handle,
    net::key::SecretKey,
    node::{Node, DEFAULT_BIND_ADDR},
};

use crate::error::{IrohError as Error, Result};

#[derive(Clone, Debug)]
pub struct IrohNode {
    inner: Node<mem::Store, iroh_sync::store::memory::Store>,
    async_runtime: Handle,
}

impl IrohNode {
    pub fn new() -> Result<Self> {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("main-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| Error::Runtime(e.to_string()))?;

        let tpc = tokio_util::task::LocalPoolHandle::new(num_cpus::get());
        let rt = iroh::bytes::util::runtime::Handle::new(tokio_rt.handle().clone(), tpc);

        let db = mem::Store::new(rt.clone());
        let sync_store = iroh_sync::store::memory::Store::default();
        let secret_key = SecretKey::generate();
        let node = rt
            .main()
            .block_on(async {
                Node::builder(db, sync_store)
                    .bind_addr(DEFAULT_BIND_ADDR.into())
                    .secret_key(secret_key)
                    .runtime(&rt)
                    .spawn()
                    .await
            })
            .map_err(|e| Error::NodeCreate(e.to_string()))?;

        Ok(IrohNode {
            inner: node,
            async_runtime: rt,
        })
    }

    pub fn peer_id(&self) -> String {
        self.inner.peer_id().to_string()
    }

    pub fn async_runtime(&self) -> &Handle {
        &self.async_runtime
    }

    pub fn inner(&self) -> &Node<mem::Store, iroh_sync::store::memory::Store> {
        &self.inner
    }
}
