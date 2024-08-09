use std::pin::Pin;

use futures::{Sink, SinkExt, StreamExt};
use iroh::client::gossip::{SubscribeResponse, SubscribeUpdate};
use iroh::gossip::net::GossipEvent;
use iroh::net::NodeId;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, UnknownReturnValue};
use napi_derive::napi;
use tokio::sync::Mutex;
use tracing::warn;

use crate::node::Iroh;

/// Gossip message
#[derive(Debug, Default)]
#[napi(object)]
pub struct Message {
    /// We have a new, direct neighbor in the swarm membership layer for this topic
    pub neighbor_up: Option<String>,
    /// We dropped direct neighbor in the swarm membership layer for this topic
    pub neighbor_down: Option<String>,
    /// A gossip message was received for this topic
    pub received: Option<MessageContent>,
    pub joined: Option<Vec<String>>,
    /// We missed some messages
    pub lagged: bool,
    /// There was a gossip error
    pub error: Option<String>,
}

/// The actual content of a gossip message.
#[derive(Debug)]
#[napi(object)]
pub struct MessageContent {
    /// The content of the message
    pub content: Vec<u8>,
    /// The node that delivered the message. This is not the same as the original author.
    pub delivered_from: String,
}

impl From<anyhow::Result<SubscribeResponse>> for Message {
    fn from(event: anyhow::Result<SubscribeResponse>) -> Self {
        match event {
            Ok(SubscribeResponse::Gossip(GossipEvent::NeighborUp(n))) => Message {
                neighbor_up: Some(n.to_string()),
                ..Default::default()
            },
            Ok(SubscribeResponse::Gossip(GossipEvent::NeighborDown(n))) => Message {
                neighbor_down: Some(n.to_string()),
                ..Default::default()
            },
            Ok(SubscribeResponse::Gossip(GossipEvent::Received(iroh::gossip::net::Message {
                content,
                delivered_from,
                ..
            }))) => Message {
                received: Some(MessageContent {
                    content: content.to_vec(),
                    delivered_from: delivered_from.to_string(),
                }),
                ..Default::default()
            },
            Ok(SubscribeResponse::Gossip(GossipEvent::Joined(nodes))) => Message {
                joined: Some(nodes.into_iter().map(|n| n.to_string()).collect()),
                ..Default::default()
            },
            Ok(SubscribeResponse::Lagged) => Message {
                lagged: true,
                ..Default::default()
            },
            Err(err) => Message {
                error: Some(err.to_string()),
                ..Default::default()
            },
        }
    }
}

/// Iroh gossip client.
#[napi]
pub struct Gossip {
    node: Iroh,
}

#[napi]
impl Iroh {
    /// Access to gossip specific funtionaliy.
    #[napi(getter)]
    pub fn gossip(&self) -> Gossip {
        Gossip { node: self.clone() }
    }
}

impl Gossip {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[napi]
impl Gossip {
    #[napi]
    pub async fn subscribe(
        &self,
        topic: Vec<u8>,
        bootstrap: Vec<String>,
        cb: ThreadsafeFunction<Message, UnknownReturnValue>,
    ) -> Result<Sender> {
        if topic.len() != 32 {
            return Err(anyhow::anyhow!("topic must not be longer than 32 bytes").into());
        }
        let topic_bytes: [u8; 32] = topic.try_into().unwrap();

        let bootstrap = bootstrap
            .into_iter()
            .map(|b| b.parse().map_err(anyhow::Error::from))
            .collect::<anyhow::Result<Vec<NodeId>>>()?;

        let (sink, mut stream) = self
            .client()
            .gossip()
            .subscribe(topic_bytes, bootstrap)
            .await?;

        tokio::task::spawn(async move {
            while let Some(event) = stream.next().await {
                let message: Message = event.into();
                if let Err(err) = cb.call_async(Ok(message)).await {
                    warn!("cb error, gossip: {:?}", err);
                }
            }
        });

        let sender = Sender(Mutex::new(Box::pin(sink)));

        Ok(sender)
    }
}

/// Gossip sender
#[napi]
pub struct Sender(Mutex<Pin<Box<dyn Sink<SubscribeUpdate, Error = anyhow::Error> + Sync + Send>>>);

#[napi]
impl Sender {
    /// Broadcast a message to all nodes in the swarm
    #[napi]
    pub async fn broadcast(&self, msg: Vec<u8>) -> Result<()> {
        self.0
            .lock()
            .await
            .send(SubscribeUpdate::Broadcast(msg.into()))
            .await?;
        Ok(())
    }

    /// Broadcast a message to all direct neighbors.
    #[napi]
    pub async fn broadcast_neighbors(&self, msg: Vec<u8>) -> Result<()> {
        self.0
            .lock()
            .await
            .send(SubscribeUpdate::BroadcastNeighbors(msg.into()))
            .await?;
        Ok(())
    }
}
