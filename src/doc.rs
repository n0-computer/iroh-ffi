use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};
use iroh::{
    bytes::util::runtime::Handle,
    client::Doc as ClientDoc,
    rpc_protocol::{ProviderRequest, ProviderResponse, ShareMode},
};

use quic_rpc::transport::flume::FlumeConnection;

use crate::{block_on, Hash, IrohError, SubscribeCallback};

/// A representation of a mutable, synchronizable key-value store.
pub struct Doc {
    pub(crate) inner: ClientDoc<FlumeConnection<ProviderResponse, ProviderRequest>>,
    pub(crate) rt: Handle,
}

impl Doc {
    /// Get the document id of this doc.
    pub fn id(&self) -> Arc<NamespaceId> {
        Arc::new(self.inner.id().into())
    }

    /// Close the document.
    pub async fn close(&self) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner.close().await.map_err(IrohError::doc)
        })
    }

    pub fn keys(&self) -> Result<Vec<Arc<Entry>>, IrohError> {
        let latest = block_on(&self.rt, async {
            let get_result = self
                .inner
                .get_many(iroh::sync::store::GetFilter::All)
                .await?;
            get_result
                .map_ok(|e| Arc::new(Entry(e)))
                .try_collect::<Vec<_>>()
                .await
        })
        .map_err(IrohError::doc)?;
        Ok(latest)
    }

    pub fn share_write(&self) -> Result<Arc<DocTicket>, IrohError> {
        block_on(&self.rt, async {
            let ticket = self
                .inner
                .share(ShareMode::Write)
                .await
                .map_err(IrohError::doc)?;

            Ok(Arc::new(DocTicket(ticket)))
        })
    }

    pub fn share_read(&self) -> Result<Arc<DocTicket>, IrohError> {
        block_on(&self.rt, async {
            let ticket = self
                .inner
                .share(ShareMode::Read)
                .await
                .map_err(IrohError::doc)?;

            Ok(Arc::new(DocTicket(ticket)))
        })
    }

    pub fn set_bytes(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Arc<Hash>, IrohError> {
        block_on(&self.rt, async {
            let hash = self
                .inner
                .set_bytes(author_id.0.clone(), key, value)
                .await
                .map_err(IrohError::doc)?;
            Ok(Arc::new(Hash(hash)))
        })
    }

    pub fn get_content_bytes(&self, entry: Arc<Entry>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.rt, async {
            let content = self
                .inner
                .read_to_bytes(&entry.0)
                .await
                .map_err(IrohError::doc)?;

            Ok(content.to_vec())
        })
    }

    pub fn stop_sync(&self) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner.leave().await.map_err(IrohError::doc)?;
            Ok(())
        })
    }

    pub fn status(&self) -> Result<Arc<OpenState>, IrohError> {
        block_on(&self.rt, async {
            let status = self
                .inner
                .status()
                .await
                .map(|s| Arc::new(OpenState(s)))
                .map_err(IrohError::doc)?;
            Ok(status)
        })
    }

    pub fn subscribe(&self, cb: Box<dyn SubscribeCallback>) -> Result<(), IrohError> {
        let client = self.inner.clone();
        self.rt.main().spawn(async move {
            let mut sub = client.subscribe().await.unwrap();
            while let Some(event) = sub.next().await {
                match event {
                    Ok(event) => {
                        if let Err(err) = cb.event(Arc::new(event.into())) {
                            println!("cb error: {:?}", err);
                        }
                    }
                    Err(err) => {
                        println!("rpc error: {:?}", err);
                    }
                }
            }
        });

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct OpenState(pub(crate) iroh::sync::actor::OpenState);
impl OpenState {}

#[derive(Debug, Clone)]
pub struct Entry(pub(crate) iroh::sync::sync::Entry);

impl From<iroh::sync::sync::Entry> for Entry {
    fn from(e: iroh::sync::sync::Entry) -> Self {
        Entry(e)
    }
}

impl Entry {
    pub fn author(&self) -> Arc<AuthorId> {
        Arc::new(AuthorId(self.0.id().author()))
    }

    pub fn key(&self) -> Vec<u8> {
        self.0.id().key().to_vec()
    }

    pub fn hash(&self) -> Arc<Hash> {
        Arc::new(Hash(self.0.content_hash()))
    }

    pub fn namespace(&self) -> Arc<NamespaceId> {
        Arc::new(NamespaceId(self.0.id().namespace()))
    }
}

pub struct AuthorId(pub(crate) iroh::sync::sync::AuthorId);

impl AuthorId {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct NamespaceId(pub(crate) iroh::sync::sync::NamespaceId);

impl From<iroh::sync::sync::NamespaceId> for NamespaceId {
    fn from(id: iroh::sync::sync::NamespaceId) -> Self {
        NamespaceId(id)
    }
}

impl NamespaceId {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub struct DocTicket(pub(crate) iroh::rpc_protocol::DocTicket);

impl DocTicket {
    pub fn from_string(content: String) -> Result<Self, IrohError> {
        let ticket = content
            .parse::<iroh::rpc_protocol::DocTicket>()
            .map_err(IrohError::doc_ticket)?;
        Ok(DocTicket(ticket))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
