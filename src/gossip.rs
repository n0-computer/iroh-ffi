
use std::sync::Arc;

use tokio::sync::Mutex;


use iroh_gossip::{Gossip, TopicId};
use iroh_gossip::api::{GossipSender, GossipReceiver, Event};
use n0_future::StreamExt;


use iroh::address_lookup::memory::MemoryLookup;



use crate::endpoint::Endpoint;
use crate::key::EndpointId;
use crate::net::EndpointAddr;
use crate::IrohError;



/// A message received on a gossip topic.
#[derive(uniffi::Record)]
pub struct GossipMessage {
    /// The endpoint that delivered this message.
    ///
    /// Note this is the peer we received the message *from*, which for gossiped
    /// messages is not necessarily the peer that originally broadcast it.
    pub sender: Arc<EndpointId>,
    pub content: Vec<u8>,
}

#[derive(uniffi::Object)]
pub struct GossipTopic {
    sender: GossipSender,
    receiver: Arc<Mutex<GossipReceiver>>,
}

#[uniffi::export]
impl GossipTopic {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn broadcast(&self, message: Vec<u8>) -> Result<(), IrohError> {
        self.sender.broadcast(message.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn next_message(&self) -> Result<Option<GossipMessage>, IrohError> {
        let mut rx = self.receiver.lock().await;
        while let Some(event_res) = rx.next().await {
            match event_res {
                Ok(Event::Received(msg)) => {
                    return Ok(Some(GossipMessage {
                        sender: Arc::new(msg.delivered_from.into()),
                        content: msg.content.to_vec(),
                    }))
                }
                Ok(_) => continue,
                Err(e) => return Err(anyhow::anyhow!(e).into()),
            }
        }
        Ok(None)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn wait_to_join(&self) -> Result<(), IrohError> {
        let mut rx = self.receiver.lock().await;
        rx.joined().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}

/// The gossip protocol handler.
///
/// This only spawns the protocol; it does not accept inbound connections on its
/// own. Register it with an [`AppRouterBuilder`](crate::AppRouterBuilder) via
/// `accept_gossip` and spawn the router to receive gossip traffic.
#[derive(uniffi::Object)]
pub struct GossipNode {
    pub(crate) inner: Gossip,
    endpoint: iroh::endpoint::Endpoint,
}

#[uniffi::export]
impl GossipNode {
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn spawn(endpoint: Arc<Endpoint>) -> Result<Self, IrohError> {
        let raw_endpoint = endpoint.raw().clone();

        let gossip = Gossip::builder().spawn(raw_endpoint.clone());

        Ok(Self { inner: gossip, endpoint: raw_endpoint })
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn subscribe(&self, topic_bytes: Vec<u8>, bootstrap_peers: Vec<Arc<EndpointAddr>>) -> Result<Arc<GossipTopic>, IrohError> {
        if topic_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Topic must be exactly 32 bytes").into());
        }
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&topic_bytes);
        let topic_id = TopicId::from_bytes(bytes);
        
        let mut peer_ids = Vec::new();
        let memory_lookup = MemoryLookup::new();

        
        for ffi_addr in bootstrap_peers {
            let addr: iroh::EndpointAddr = (*ffi_addr).clone().try_into()
                .map_err(|e| anyhow::anyhow!("Invalid addr: {:?}", e))?;
            
            memory_lookup.add_endpoint_info(addr.clone());
            peer_ids.push(addr.id);
        }
        
        
        if !peer_ids.is_empty() {
            if let Ok(lookup) = self.endpoint.address_lookup() {
                lookup.add(memory_lookup);
            }
        }
        
        let topic = self.inner
            .subscribe(topic_id, peer_ids)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let (sender, receiver) = topic.split();

        Ok(Arc::new(GossipTopic {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }))
    }
}