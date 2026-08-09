
use std::sync::Arc;

use tokio::sync::Mutex;


use iroh_gossip::{Gossip, TopicId};
use iroh_gossip::api::{GossipSender, GossipReceiver, Event};
use n0_future::StreamExt;


use iroh::address_lookup::memory::MemoryLookup;



use crate::endpoint::Endpoint;
use crate::net::EndpointAddr;
use crate::IrohError;



#[derive(uniffi::Object)]
pub struct GossipTopic {
    sender: GossipSender,
    receiver: Arc<Mutex<GossipReceiver>>,
}

#[uniffi::export]
impl GossipTopic {
    pub async fn broadcast(&self, message: Vec<u8>) -> Result<(), IrohError> {
        self.sender.broadcast(message.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    pub async fn next_message(&self) -> Result<Option<Vec<u8>>, IrohError> {
        let mut rx = self.receiver.lock().await;
        while let Some(event_res) = rx.next().await {
            match event_res {
                Ok(Event::Received(msg)) => return Ok(Some(msg.content.to_vec())),
                Ok(_) => continue, 
                Err(e) => return Err(anyhow::anyhow!(e).into()),
            }
        }
        Ok(None)
    }

    // Add this missing method!
    pub async fn wait_to_join(&self) -> Result<(), IrohError> {
        let mut rx = self.receiver.lock().await;
        rx.joined().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}

#[derive(uniffi::Object)]
pub struct GossipNode {
    pub inner: Gossip,
    endpoint: iroh::endpoint::Endpoint, 
    // router: Router,
}

#[uniffi::export]
impl GossipNode {
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn spawn(endpoint: Arc<Endpoint>) -> Result<Self, IrohError> {
        let raw_endpoint = endpoint.raw().clone();
        
        let gossip = Gossip::builder().spawn(raw_endpoint.clone());

        // let router = Router::builder(raw_endpoint.clone())
        //     .accept(ALPN, gossip.clone())
        //     .spawn();


        Ok(Self { inner: gossip, endpoint: raw_endpoint })
    }

    
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