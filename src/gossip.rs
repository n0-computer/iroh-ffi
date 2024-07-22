use std::pin::Pin;
use std::sync::Arc;

use futures::{Sink, SinkExt, StreamExt};
use iroh::client::gossip::{SubscribeResponse, SubscribeUpdate};
use iroh::gossip::dispatcher::GossipEvent;
use iroh::net::NodeId;
use tokio::sync::Mutex;
use tracing::warn;

use crate::node::Iroh;
use crate::{CallbackError, IrohError};

/// Gossip message
#[derive(Debug, uniffi::Object)]
pub enum Message {
    /// We have a new, direct neighbor in the swarm membership layer for this topic
    NeighborUp(String),
    /// We dropped direct neighbor in the swarm membership layer for this topic
    NeighborDown(String),
    /// A gossip message was received for this topic
    Received {
        /// The content of the message
        content: Vec<u8>,
        /// The node that delivered the message. This is not the same as the original author.
        delivered_from: String,
    },
    /// We missed some messages
    Lagged,
    /// There was a gossip error
    Error(String),
}

#[derive(Debug, uniffi::Enum)]
pub enum MessageType {
    NeighborUp,
    NeighborDown,
    Received,
    Lagged,
    Error,
}

#[uniffi::export]
impl Message {
    pub fn r#type(&self) -> MessageType {
        match self {
            Self::NeighborUp(_) => MessageType::NeighborUp,
            Self::NeighborDown(_) => MessageType::NeighborDown,
            Self::Received { .. } => MessageType::Received,
            Self::Lagged => MessageType::Lagged,
            Self::Error(_) => MessageType::Error,
        }
    }

    pub fn as_neighbor_up(&self) -> String {
        if let Self::NeighborUp(s) = self {
            s.clone()
        } else {
            panic!("not a NeighborUp message");
        }
    }

    pub fn as_neighbor_down(&self) -> String {
        if let Self::NeighborDown(s) = self {
            s.clone()
        } else {
            panic!("not a NeighborDown message");
        }
    }

    pub fn as_received(&self) -> MessageContent {
        if let Self::Received {
            content,
            delivered_from,
        } = self
        {
            MessageContent {
                content: content.clone(),
                delivered_from: delivered_from.clone(),
            }
        } else {
            panic!("not a Received message");
        }
    }

    pub fn as_error(&self) -> String {
        if let Self::Error(s) = self {
            s.clone()
        } else {
            panic!("not a Error message");
        }
    }
}

/// The actual content of a gossip message.
#[derive(Debug, uniffi::Record)]
pub struct MessageContent {
    /// The content of the message
    pub content: Vec<u8>,
    /// The node that delivered the message. This is not the same as the original author.
    pub delivered_from: String,
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait GossipMessageCallback: Send + Sync + 'static {
    async fn on_message(&self, msg: Arc<Message>) -> Result<(), CallbackError>;
}

/// Iroh gossip client.
#[derive(uniffi::Object)]
pub struct Gossip {
    node: Iroh,
}

#[uniffi::export]
impl Iroh {
    /// Access to gossip specific funtionaliy.
    pub fn gossip(&self) -> Gossip {
        Gossip { node: self.clone() }
    }
}

impl Gossip {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[uniffi::export]
impl Gossip {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn subscribe(
        &self,
        topic: Vec<u8>,
        bootstrap: Vec<String>,
        cb: Arc<dyn GossipMessageCallback>,
    ) -> Result<Sender, IrohError> {
        if topic.len() != 32 {
            return Err(anyhow::anyhow!("topic must not be longer than 32 bytes").into());
        }
        let topic_bytes: [u8; 32] = topic.try_into().unwrap();

        let bootstrap = bootstrap
            .into_iter()
            .map(|b| b.parse())
            .collect::<Result<Vec<NodeId>, _>>()
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let (sink, mut stream) = self
            .client()
            .gossip()
            .subscribe(topic_bytes, bootstrap)
            .await?;

        tokio::task::spawn(async move {
            while let Some(event) = stream.next().await {
                let message = match event {
                    Ok(SubscribeResponse::Gossip(GossipEvent::NeighborUp(n))) => {
                        Message::NeighborUp(n.to_string())
                    }
                    Ok(SubscribeResponse::Gossip(GossipEvent::NeighborDown(n))) => {
                        Message::NeighborDown(n.to_string())
                    }
                    Ok(SubscribeResponse::Gossip(GossipEvent::Received(
                        iroh::gossip::dispatcher::Message {
                            content,
                            delivered_from,
                            ..
                        },
                    ))) => Message::Received {
                        content: content.to_vec(),
                        delivered_from: delivered_from.to_string(),
                    },
                    Ok(SubscribeResponse::Lagged) => Message::Lagged,
                    Err(err) => Message::Error(err.to_string()),
                };
                if let Err(err) = cb.on_message(Arc::new(message)).await {
                    warn!("cb error, gossip: {:?}", err);
                }
            }
        });

        let sender = Sender(Mutex::new(Box::pin(sink)));

        Ok(sender)
    }
}

/// Gossip sender
#[derive(uniffi::Object)]
pub struct Sender(Mutex<Pin<Box<dyn Sink<SubscribeUpdate, Error = anyhow::Error> + Sync + Send>>>);

#[uniffi::export]
impl Sender {
    /// Broadcast a message to all nodes in the swarm
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn broadcast(&self, msg: Vec<u8>) -> Result<(), IrohError> {
        self.0
            .lock()
            .await
            .send(SubscribeUpdate::Broadcast(msg.into()))
            .await?;
        Ok(())
    }

    /// Broadcast a message to all direct neighbors.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn broadcast_neighbors(&self, msg: Vec<u8>) -> Result<(), IrohError> {
        self.0
            .lock()
            .await
            .send(SubscribeUpdate::BroadcastNeighbors(msg.into()))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    #[tokio::test]
    async fn test_gossip_basic() {
        let n0 = Iroh::memory().await.unwrap();
        let n1 = Iroh::memory().await.unwrap();

        struct Cb {
            channel: mpsc::Sender<Arc<Message>>,
        }
        #[async_trait::async_trait]
        impl GossipMessageCallback for Cb {
            async fn on_message(&self, message: Arc<Message>) -> Result<(), CallbackError> {
                println!("<< {:?}", message);
                self.channel.send(message).await.unwrap();
                Ok(())
            }
        }

        let topic = [1u8; 32].to_vec();

        let (sender0, _receiver0) = mpsc::channel(8);
        let cb0 = Cb { channel: sender0 };
        let n1_id = n1.node().node_id().await.unwrap();
        let n1_addr = n1.node().node_addr().await.unwrap();
        n0.node().add_node_addr(&n1_addr).await.unwrap();

        let sink0 = n0
            .gossip()
            .subscribe(topic.clone(), vec![n1_id.to_string()], Arc::new(cb0))
            .await
            .unwrap();

        let (sender1, mut receiver1) = mpsc::channel(8);
        let cb1 = Cb { channel: sender1 };
        let n0_id = n0.node().node_id().await.unwrap();
        let n0_addr = n0.node().node_addr().await.unwrap();
        n1.node().add_node_addr(&n0_addr).await.unwrap();
        let _ = n1
            .gossip()
            .subscribe(topic.clone(), vec![n0_id.to_string()], Arc::new(cb1))
            .await
            .unwrap();

        // Send message on n0
        println!("sending message");
        let msg_content = b"hello";
        sink0.broadcast(msg_content.to_vec()).await.unwrap();

        // Receive on n1
        let recv_fut = async {
            loop {
                let Some(event) = receiver1.recv().await else {
                    panic!("receiver stream closed before receiving gossip message");
                };
                println!("event: {:?}", event);
                if let Message::Received {
                    ref content,
                    ref delivered_from,
                } = &*event
                {
                    assert_eq!(content, msg_content);
                    assert_eq!(delivered_from, &n0_id.to_string());

                    break;
                }
            }
        };
        tokio::time::timeout(std::time::Duration::from_secs(10), recv_fut)
            .await
            .expect("timeout reached and no gossip message received");
    }
}
