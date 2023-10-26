use std::str::FromStr;
use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};
use iroh::{
    bytes::util::runtime::Handle,
    client::Doc as ClientDoc,
    rpc_protocol::{ProviderRequest, ProviderResponse},
};

use quic_rpc::transport::flume::FlumeConnection;

use crate::{block_on, Hash, IrohError, PublicKey, SocketAddr, SocketAddrType, SubscribeCallback};

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
    pub fn close(&self) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner.close().await.map_err(IrohError::doc)
        })
    }

    /// Set the content of a key to a byte array.
    pub fn set_bytes(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Arc<Hash>, IrohError> {
        block_on(&self.rt, async {
            let hash = self
                .inner
                .set_bytes(author_id.0, key, value)
                .await
                .map_err(IrohError::doc)?;
            Ok(Arc::new(Hash(hash)))
        })
    }

    /// Set an entries on the doc via its key, hash, and size.
    pub fn set_hash(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
        hash: Arc<Hash>,
        size: u64,
    ) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner
                .set_hash(author_id.0, key, hash.0, size)
                .await
                .map_err(IrohError::doc)?;
            Ok(())
        })
    }

    /// Get the content size of an [`Entry`]
    pub fn size(&self, entry: Arc<Entry>) -> Result<u64, IrohError> {
        block_on(&self.rt, async {
            let r = self.inner.read(&entry.0).await.map_err(IrohError::doc)?;
            Ok(r.size())
        })
    }

    /// Read all content of an [`Entry`] into a buffer.
    /// This allocates a buffer for the full entry. Use only if you know that the entry you're
    /// reading is small. If not sure, use [`Self::size`] and check the size with
    /// before calling [`Self::read_to_bytes`].
    pub fn read_to_bytes(&self, entry: Arc<Entry>) -> Result<Vec<u8>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .read_to_bytes(&entry.0)
                .await
                .map(|c| c.to_vec())
                .map_err(IrohError::doc)
        })
    }

    /// Delete entries that match the given `author` and key `prefix`.
    ///
    /// This inserts an empty entry with the key set to `prefix`, effectively clearing all other
    /// entries whose key starts with or is equal to the given `prefix`.
    ///
    /// Returns the number of entries deleted.
    pub fn del(&self, author_id: Arc<AuthorId>, prefix: Vec<u8>) -> Result<u64, IrohError> {
        block_on(&self.rt, async {
            let num_del = self
                .inner
                .del(author_id.0, prefix)
                .await
                .map_err(IrohError::doc)?;
            u64::try_from(num_del).map_err(IrohError::doc)
        })
    }

    /// Get the latest entry for a key and author.
    pub fn get_one(
        &self,
        author_id: Arc<AuthorId>,
        key: Vec<u8>,
    ) -> Result<Option<Arc<Entry>>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .get_one(author_id.0, key)
                .await
                .map(|e| e.map(|e| Arc::new(e.into())))
                .map_err(IrohError::doc)
        })
    }

    /// Get entries.
    ///
    /// Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn get_many(&self, filter: Arc<GetFilter>) -> Result<Vec<Arc<Entry>>, IrohError> {
        block_on(&self.rt, async {
            let entries = self
                .inner
                .get_many((*filter).clone().into())
                .await
                .map_err(IrohError::doc)?
                .map_ok(|e| Arc::new(Entry(e)))
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::doc)?;
            Ok(entries)
        })
    }

    /// Share this document with peers over a ticket.
    pub fn share(&self, mode: ShareMode) -> anyhow::Result<Arc<DocTicket>, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .share(mode.into())
                .await
                .map(|ticket| Arc::new(DocTicket(ticket)))
                .map_err(IrohError::doc)
        })
    }

    /// Start to sync this document with a list of peers.
    pub fn start_sync(&self, peers: Vec<Arc<PeerAddr>>) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner
                .start_sync(peers.into_iter().map(|p| (*p).clone().into()).collect())
                .await
                .map_err(IrohError::doc)
        })
    }

    /// Stop the live sync for this document.
    pub fn leave(&self) -> Result<(), IrohError> {
        block_on(&self.rt, async {
            self.inner.leave().await.map_err(IrohError::doc)
        })
    }

    /// Get status info for this document
    pub fn status(&self) -> Result<OpenState, IrohError> {
        block_on(&self.rt, async {
            self.inner
                .status()
                .await
                .map(|o| o.into())
                .map_err(IrohError::doc)
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

/// The state for an open replica.
#[derive(Debug, Clone, Copy)]
pub struct OpenState {
    /// Whether to accept sync requests for this replica.
    pub sync: bool,
    /// How many event subscriptions are open
    pub subscribers: u64,
    /// By how many handles the replica is currently held open
    pub handles: u64,
}

impl From<iroh::sync::actor::OpenState> for OpenState {
    fn from(value: iroh::sync::actor::OpenState) -> Self {
        OpenState {
            sync: value.sync,
            subscribers: value.subscribers as u64,
            handles: value.handles as u64,
        }
    }
}

/// A peer and it's addressing information.
#[derive(Debug, Clone)]
pub struct PeerAddr {
    peer_id: Arc<PublicKey>,
    derp_region: Option<u16>,
    addresses: Vec<Arc<SocketAddr>>,
}

impl PeerAddr {
    /// Create a new [`PeerAddr`] with empty [`AddrInfo`].
    pub fn new(
        peer_id: Arc<PublicKey>,
        derp_region: Option<u16>,
        addresses: Vec<Arc<SocketAddr>>,
    ) -> Self {
        Self {
            peer_id,
            derp_region,
            addresses,
        }
    }

    /// Get the direct addresses of this peer.
    pub fn direct_addresses(&self) -> Vec<Arc<SocketAddr>> {
        self.addresses.clone()
    }

    /// Get the derp region of this peer.
    pub fn derp_region(&self) -> Option<u16> {
        self.derp_region
    }
}

impl From<PeerAddr> for iroh::net::magic_endpoint::PeerAddr {
    fn from(value: PeerAddr) -> Self {
        let mut peer_addr = iroh::net::magic_endpoint::PeerAddr::new(value.peer_id.0);
        let addresses = value.direct_addresses().into_iter().map(|addr| {
            let typ = addr.r#type();
            match typ {
                SocketAddrType::V4 => {
                    let addr_str = addr.to_string();
                    std::net::SocketAddrV4::from_str(&addr_str)
                        .expect("checked")
                        .into()
                }
                SocketAddrType::V6 => {
                    let addr_str = addr.to_string();
                    std::net::SocketAddrV6::from_str(&addr_str)
                        .expect("checked")
                        .into()
                }
            }
        });
        if let Some(derp_region) = value.derp_region() {
            peer_addr = peer_addr.with_derp_region(derp_region);
        }
        peer_addr = peer_addr.with_direct_addresses(addresses);
        peer_addr
    }
}

/// Intended capability for document share tickets
#[derive(Debug)]
pub enum ShareMode {
    /// Read-only access
    Read,
    /// Write access
    Write,
}

impl From<ShareMode> for iroh::rpc_protocol::ShareMode {
    fn from(mode: ShareMode) -> Self {
        match mode {
            ShareMode::Read => iroh::rpc_protocol::ShareMode::Read,
            ShareMode::Write => iroh::rpc_protocol::ShareMode::Write,
        }
    }
}

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

#[derive(Debug, Clone)]
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

/// Filter a get query onto a namespace
#[derive(Debug, Clone)]
pub enum GetFilter {
    /// No filter, list all entries
    All,
    /// Filter for exact key match
    Key(Vec<u8>),
    /// Filter for key prefix
    Prefix(Vec<u8>),
    /// Filter by author
    Author(Arc<AuthorId>),
    /// Filter by key prefix and author
    AuthorAndPrefix(Arc<AuthorId>, Vec<u8>),
}

use iroh::sync::store::GetFilter as IrohGetFilter;

impl From<GetFilter> for IrohGetFilter {
    fn from(filter: GetFilter) -> Self {
        match filter {
            GetFilter::All => IrohGetFilter::All,
            GetFilter::Key(key) => IrohGetFilter::Key(key),
            GetFilter::Prefix(prefix) => IrohGetFilter::Prefix(prefix),
            GetFilter::Author(author_id) => IrohGetFilter::Author(author_id.0),
            GetFilter::AuthorAndPrefix(author_id, prefix) => {
                IrohGetFilter::AuthorAndPrefix(author_id.0, prefix)
            }
        }
    }
}

impl GetFilter {
    /// Filter by [`AuthorId`] and prefix
    pub fn author_prefix(author: Arc<AuthorId>, prefix: Vec<u8>) -> Self {
        GetFilter::AuthorAndPrefix(author, prefix)
    }

    /// No filter, get all entries in a namespace
    pub fn all() -> Self {
        GetFilter::All
    }

    /// Filter by [`AuthorId`]
    pub fn author(author: Arc<AuthorId>) -> Self {
        GetFilter::Author(author)
    }

    /// Filter by prefix
    pub fn prefix(prefix: Vec<u8>) -> Self {
        GetFilter::Prefix(prefix)
    }

    /// Filter by an exact key
    pub fn key(key: Vec<u8>) -> Self {
        GetFilter::Key(key)
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
