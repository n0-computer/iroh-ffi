use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

// 1. Correct trait is ProtocolHandler, and it uses async fn directly
use iroh::protocol::{Router, RouterBuilder, ProtocolHandler, AcceptError};
// 2. We receive a fully established Connection from the router
use iroh::endpoint::Connection as IrohConnection;

use crate::endpoint::Endpoint; 
// Import the FFI Connection wrapper (adjust to crate::endpoint::Connection if your FFI puts it there)
use crate::Connection;
use crate::gossip::GossipNode;
use crate::IrohError;

#[derive(Clone, Debug)]
struct PythonStreamHandler {
    tx: mpsc::Sender<IrohConnection>,
}

// 3. Implement ProtocolHandler using the native async trait signature
impl ProtocolHandler for PythonStreamHandler {
    async fn accept(&self, conn: IrohConnection) -> Result<(), AcceptError> {
        // Send the fully established connection to our Python receiver channel
        let _ = self.tx.send(conn).await;
        Ok(())
    }
}

#[derive(uniffi::Object)]
pub struct AppRouterBuilder {
    builder: Mutex<Option<RouterBuilder>>,
}

#[uniffi::export]
impl AppRouterBuilder {
    #[uniffi::constructor]
    pub fn new(endpoint: Arc<Endpoint>) -> Self {
        let raw_ep = endpoint.raw().clone();
        Self {
            builder: Mutex::new(Some(Router::builder(raw_ep))),
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_gossip(&self, gossip: Arc<GossipNode>) -> Result<(), IrohError> {
        let mut lock = self.builder.lock().await;
        if let Some(builder) = lock.take() {
            *lock = Some(builder.accept(iroh_gossip::ALPN, gossip.inner.clone()));
            Ok(())
        } else {
            Err(anyhow::anyhow!("Builder already consumed").into())
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_custom_alpn(&self, alpn: Vec<u8>) -> Result<Arc<CustomAlpnReceiver>, IrohError> {
        let (tx, rx) = mpsc::channel(32);
        let handler = PythonStreamHandler { tx };

        let mut lock = self.builder.lock().await;
        if let Some(builder) = lock.take() {
            *lock = Some(builder.accept(alpn, handler));
            Ok(Arc::new(CustomAlpnReceiver {
                rx: Mutex::new(rx)
            }))
        } else {
            Err(anyhow::anyhow!("Builder already consumed").into())
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn spawn(&self) -> Result<Arc<AppRouter>, IrohError> {
        let mut lock = self.builder.lock().await;
        if let Some(builder) = lock.take() {
            // REMOVED .await and .map_err()
            let router = builder.spawn(); 
            Ok(Arc::new(AppRouter { inner: router }))
        } else {
            Err(anyhow::anyhow!("Router already spawned").into())
        }
    }
}

#[derive(uniffi::Object)]
pub struct CustomAlpnReceiver {
    rx: Mutex<mpsc::Receiver<IrohConnection>>,
}

#[uniffi::export]
impl CustomAlpnReceiver {
    /// Yields the fully connected QUIC connection for Python
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn next_connection(&self) -> Result<Option<Arc<Connection>>, IrohError> {
        let mut rx = self.rx.lock().await;
        if let Some(conn) = rx.recv().await {
            // Wrap the raw iroh connection directly into the FFI struct
            Ok(Some(Arc::new(Connection::from(conn))))
        } else {
            Ok(None)
        }
    }
}

#[derive(uniffi::Object)]
pub struct AppRouter {
    inner: Router,
}

#[uniffi::export]
impl AppRouter {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn shutdown(&self) -> Result<(), IrohError> {
        self.inner.shutdown().await.map_err(|e| anyhow::anyhow!(e).into())
    }
}