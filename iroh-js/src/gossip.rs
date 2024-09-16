use std::pin::Pin;

use futures::{Sink, SinkExt, StreamExt};
use iroh::client::gossip::{SubscribeResponse, SubscribeUpdate};
use iroh::gossip::net::GossipEvent;
use iroh::net::NodeId;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
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

impl From<SubscribeResponse> for Message {
    fn from(event: SubscribeResponse) -> Self {
        match event {
            SubscribeResponse::Gossip(GossipEvent::NeighborUp(n)) => Message {
                neighbor_up: Some(n.to_string()),
                ..Default::default()
            },
            SubscribeResponse::Gossip(GossipEvent::NeighborDown(n)) => Message {
                neighbor_down: Some(n.to_string()),
                ..Default::default()
            },
            SubscribeResponse::Gossip(GossipEvent::Received(iroh::gossip::net::Message {
                content,
                delivered_from,
                ..
            })) => Message {
                received: Some(MessageContent {
                    content: content.to_vec(),
                    delivered_from: delivered_from.to_string(),
                }),
                ..Default::default()
            },
            SubscribeResponse::Gossip(GossipEvent::Joined(nodes)) => Message {
                joined: Some(nodes.into_iter().map(|n| n.to_string()).collect()),
                ..Default::default()
            },
            SubscribeResponse::Lagged => Message {
                lagged: true,
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
        self.node.inner_client()
    }
}

#[napi]
impl Gossip {
    #[napi]
    pub async fn subscribe(
        &self,
        topic: Vec<u8>,
        bootstrap: Vec<String>,
        cb: ThreadsafeFunction<Message, ()>,
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

        let cancel_token = CancellationToken::new();
        let cancel = cancel_token.clone();
        tokio::task::spawn(async move {
            loop {
                tokio::select! {
                    biased;

                    _ = cancel_token.cancelled() => {
                        break;
                    }
                    Some(event) = stream.next() => {
                        let message: Result<Message> = event.map(Into::into).map_err(Into::into);
                        if let Err(err) = cb.call_async(message).await {
                            warn!("cb error, gossip: {:?}", err);
                        }
                    }
                    else => {
                        break;
                    }
                }
            }
        });

        let sender = Sender {
            sink: Mutex::new(Box::pin(sink)),
            cancel,
        };

        Ok(sender)
    }
}

/// Gossip sender
#[napi]
pub struct Sender {
    sink: Mutex<Pin<Box<dyn Sink<SubscribeUpdate, Error = anyhow::Error> + Sync + Send>>>,
    cancel: CancellationToken,
}

#[napi]
impl Sender {
    /// Broadcast a message to all nodes in the swarm
    #[napi]
    pub async fn broadcast(&self, msg: Vec<u8>) -> Result<()> {
        self.sink
            .lock()
            .await
            .send(SubscribeUpdate::Broadcast(msg.into()))
            .await?;
        Ok(())
    }

    /// Broadcast a message to all direct neighbors.
    #[napi]
    pub async fn broadcast_neighbors(&self, msg: Vec<u8>) -> Result<()> {
        self.sink
            .lock()
            .await
            .send(SubscribeUpdate::BroadcastNeighbors(msg.into()))
            .await?;
        Ok(())
    }

    /// Closes the subscription, it is an error to use it afterwards
    #[napi]
    pub async fn close(&self) -> Result<()> {
        if self.cancel.is_cancelled() {
            return Err(anyhow::anyhow!("already closed").into());
        }
        self.sink.lock().await.close().await?;
        self.cancel.cancel();
        Ok(())
    }
}
