use std::sync::Arc;

use iroh::net::endpoint;
use tokio::sync::Mutex;

use crate::{IrohError, NodeAddr, PublicKey};

#[derive(Clone, uniffi::Object)]
pub struct Endpoint(endpoint::Endpoint);

impl Endpoint {
    pub fn new(ep: endpoint::Endpoint) -> Self {
        Endpoint(ep)
    }
}

#[uniffi::export]
impl Endpoint {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect(
        &self,
        node_addr: &NodeAddr,
        alpn: &[u8],
    ) -> Result<Connection, IrohError> {
        let conn = self.0.connect(node_addr.clone().try_into()?, alpn).await?;
        Ok(Connection(conn))
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect_by_node_id(
        &self,
        node_id: String,
        alpn: &[u8],
    ) -> Result<Connection, IrohError> {
        let node_id: iroh::net::NodeId = node_id.parse().map_err(anyhow::Error::from)?;
        let conn = self.0.connect_by_node_id(node_id, &alpn).await?;
        Ok(Connection(conn))
    }
}

#[derive(uniffi::Object)]
pub struct Connecting(Mutex<Option<endpoint::Connecting>>);

impl Connecting {
    pub fn new(conn: endpoint::Connecting) -> Self {
        Connecting(Mutex::new(Some(conn)))
    }
}

#[uniffi::export]
impl Connecting {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn connect(&self) -> Result<Connection, IrohError> {
        match self.0.lock().await.take() {
            Some(conn) => {
                let conn = conn.await.map_err(anyhow::Error::from)?;
                Ok(Connection(conn))
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn alpn(&self) -> Result<Vec<u8>, IrohError> {
        match &mut *self.0.lock().await {
            Some(conn) => {
                let alpn = conn.alpn().await?;
                Ok(alpn)
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn local_ip(&self) -> Result<Option<String>, IrohError> {
        match &*self.0.lock().await {
            Some(conn) => {
                let ip = conn.local_ip();
                Ok(ip.map(|ip| ip.to_string()))
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn remote_address(&self) -> Result<String, IrohError> {
        match &*self.0.lock().await {
            Some(conn) => {
                let addr = conn.remote_address();
                Ok(addr.to_string())
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }
}

#[derive(uniffi::Object)]
pub struct Connection(endpoint::Connection);

#[uniffi::export]
impl Connection {
    #[uniffi::method]
    pub fn get_remote_node_id(&self) -> Result<PublicKey, IrohError> {
        let id = endpoint::get_remote_node_id(&self.0)?;
        Ok(id.into())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn open_uni(&self) -> Result<SendStream, IrohError> {
        let s = self.0.open_uni().await.map_err(anyhow::Error::from)?;
        Ok(SendStream::new(s))
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_uni(&self) -> Result<RecvStream, IrohError> {
        let r = self.0.accept_uni().await.map_err(anyhow::Error::from)?;
        Ok(RecvStream::new(r))
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn open_bi(&self) -> Result<BiStream, IrohError> {
        let (s, r) = self.0.open_bi().await.map_err(anyhow::Error::from)?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn accept_bi(&self) -> Result<BiStream, IrohError> {
        let (s, r) = self.0.accept_bi().await.map_err(anyhow::Error::from)?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_datagram(&self) -> Result<Vec<u8>, IrohError> {
        let res = self.0.read_datagram().await.map_err(anyhow::Error::from)?;
        Ok(res.to_vec())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn closed(&self) -> String {
        let err = self.0.closed().await;
        err.to_string()
    }

    #[uniffi::method]
    pub fn close_reason(&self) -> Option<String> {
        let err = self.0.close_reason();
        err.map(|s| s.to_string())
    }

    #[uniffi::method]
    pub fn close(&self, error_code: u64, reason: &[u8]) -> Result<(), IrohError> {
        let code = endpoint::VarInt::from_u64(error_code).map_err(anyhow::Error::from)?;
        self.0.close(code, reason);
        Ok(())
    }

    #[uniffi::method]
    pub fn send_datagram(&self, data: Vec<u8>) -> Result<(), IrohError> {
        self.0
            .send_datagram(data.into())
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn send_datagram_wait(&self, data: Vec<u8>) -> Result<(), IrohError> {
        self.0
            .send_datagram_wait(data.into())
            .await
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method]
    pub fn max_datagram_size(&self) -> Option<u64> {
        self.0.max_datagram_size().map(|s| s as _)
    }

    #[uniffi::method]
    pub fn datagram_send_buffer_space(&self) -> u64 {
        self.0.datagram_send_buffer_space() as _
    }

    #[uniffi::method]
    pub fn remote_address(&self) -> String {
        self.0.remote_address().to_string()
    }

    #[uniffi::method]
    pub fn local_ip(&self) -> Option<String> {
        self.0.local_ip().map(|s| s.to_string())
    }

    #[uniffi::method]
    pub fn rtt(&self) -> u64 {
        self.0.rtt().as_millis() as _
    }

    #[uniffi::method]
    pub fn stable_id(&self) -> u64 {
        self.0.stable_id() as _
    }

    #[uniffi::method]
    pub fn set_max_concurrent_uni_stream(&self, count: u64) -> Result<(), IrohError> {
        let n = endpoint::VarInt::from_u64(count).map_err(anyhow::Error::from)?;
        self.0.set_max_concurrent_uni_streams(n);
        Ok(())
    }

    #[uniffi::method]
    pub fn set_receive_window(&self, count: u64) -> Result<(), IrohError> {
        let n = endpoint::VarInt::from_u64(count).map_err(anyhow::Error::from)?;
        self.0.set_receive_window(n);
        Ok(())
    }

    #[uniffi::method]
    pub fn set_max_concurrent_bii_stream(&self, count: u64) -> Result<(), IrohError> {
        let n = endpoint::VarInt::from_u64(count).map_err(anyhow::Error::from)?;
        self.0.set_max_concurrent_bi_streams(n);
        Ok(())
    }
}

#[derive(uniffi::Object)]
pub struct BiStream {
    send: SendStream,
    recv: RecvStream,
}

#[uniffi::export]
impl BiStream {
    #[uniffi::method]
    pub fn send(&self) -> SendStream {
        self.send.clone()
    }

    #[uniffi::method]
    pub fn recv(&self) -> RecvStream {
        self.recv.clone()
    }
}

#[derive(Clone, uniffi::Object)]
pub struct SendStream(Arc<Mutex<endpoint::SendStream>>);

impl SendStream {
    fn new(s: endpoint::SendStream) -> Self {
        SendStream(Arc::new(Mutex::new(s)))
    }
}

#[uniffi::export]
impl SendStream {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn write(&self, buf: &[u8]) -> Result<u64, IrohError> {
        let mut s = self.0.lock().await;
        let written = s.write(buf).await.map_err(anyhow::Error::from)?;
        Ok(written as _)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn write_all(&self, buf: &[u8]) -> Result<(), IrohError> {
        let mut s = self.0.lock().await;
        s.write_all(&buf).await.map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn finish(&self) -> Result<(), IrohError> {
        let mut s = self.0.lock().await;
        s.finish().map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn reset(&self, error_code: u64) -> Result<(), IrohError> {
        let error_code = endpoint::VarInt::from_u64(error_code).map_err(anyhow::Error::from)?;
        let mut s = self.0.lock().await;
        s.reset(error_code).map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn set_priority(&self, p: i32) -> Result<(), IrohError> {
        let s = self.0.lock().await;
        s.set_priority(p).map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn priority(&self) -> Result<i32, IrohError> {
        let s = self.0.lock().await;
        let p = s.priority().map_err(anyhow::Error::from)?;
        Ok(p)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stopped(&self) -> Result<Option<u64>, IrohError> {
        let mut s = self.0.lock().await;
        let res = s.stopped().await.map_err(anyhow::Error::from)?;
        let res = res.map(|r| r.into_inner().into());
        Ok(res)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn id(&self) -> String {
        let r = self.0.lock().await;
        r.id().to_string()
    }
}

#[derive(Clone, uniffi::Object)]
pub struct RecvStream(Arc<Mutex<endpoint::RecvStream>>);

impl RecvStream {
    fn new(s: endpoint::RecvStream) -> Self {
        RecvStream(Arc::new(Mutex::new(s)))
    }
}

#[uniffi::export]
impl RecvStream {
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read(&self, size_limit: u32) -> Result<Vec<u8>, IrohError> {
        let mut buf = vec![0u8; size_limit as _];
        let mut r = self.0.lock().await;
        let res = r.read(&mut buf).await.map_err(anyhow::Error::from)?;
        let len = res.unwrap_or(0);
        buf.truncate(len);
        Ok(buf)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_exact(&self, size: u32) -> Result<Vec<u8>, IrohError> {
        let mut buf = vec![0u8; size as _];
        let mut r = self.0.lock().await;
        r.read_exact(&mut buf).await.map_err(anyhow::Error::from)?;
        Ok(buf)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn read_to_end(&self, size_limit: u32) -> Result<Vec<u8>, IrohError> {
        let mut r = self.0.lock().await;
        let res = r
            .read_to_end(size_limit as _)
            .await
            .map_err(anyhow::Error::from)?;
        Ok(res)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn id(&self) -> String {
        let r = self.0.lock().await;
        r.id().to_string()
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stop(&self, error_code: u64) -> Result<(), IrohError> {
        let error_code = endpoint::VarInt::from_u64(error_code).map_err(anyhow::Error::from)?;
        let mut r = self.0.lock().await;
        r.stop(error_code).map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn received_reset(&self) -> Result<Option<u64>, IrohError> {
        let mut r = self.0.lock().await;
        let code = r.received_reset().await.map_err(anyhow::Error::from)?;
        let code = code.map(|c| c.into_inner());
        Ok(code)
    }
}
