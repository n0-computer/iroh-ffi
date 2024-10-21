use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use tokio::sync::Mutex;

use iroh::net::endpoint;

use crate::{NodeAddr, PublicKey};

#[derive(Clone)]
#[napi]
pub struct Endpoint(endpoint::Endpoint);

#[napi]
impl Endpoint {
    pub fn new(ep: endpoint::Endpoint) -> Self {
        Endpoint(ep)
    }

    #[napi]
    pub async fn connect(&self, node_addr: NodeAddr, alpn: Uint8Array) -> Result<Connection> {
        let node_addr: iroh::net::NodeAddr = node_addr.try_into()?;
        let conn = self.0.connect(node_addr, &alpn).await?;
        Ok(Connection(conn))
    }
}

#[napi]
pub struct Connecting(Mutex<Option<endpoint::Connecting>>);

#[napi]
impl Connecting {
    pub fn new(conn: endpoint::Connecting) -> Self {
        Connecting(Mutex::new(Some(conn)))
    }

    #[napi]
    pub async fn connect(&self) -> Result<Connection> {
        match self.0.lock().await.take() {
            Some(conn) => {
                let conn = conn.await.map_err(anyhow::Error::from)?;
                Ok(Connection(conn))
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }

    #[napi]
    pub async fn alpn(&self) -> Result<Buffer> {
        match &mut *self.0.lock().await {
            Some(conn) => {
                let alpn = conn.alpn().await?;
                Ok(alpn.into())
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }

    #[napi]
    pub async fn local_ip(&self) -> Result<Option<String>> {
        match &*self.0.lock().await {
            Some(conn) => {
                let ip = conn.local_ip();
                Ok(ip.map(|ip| ip.to_string()))
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }

    #[napi]
    pub async fn remote_address(&self) -> Result<String> {
        match &*self.0.lock().await {
            Some(conn) => {
                let addr = conn.remote_address();
                Ok(addr.to_string())
            }
            None => Err(anyhow::anyhow!("already used").into()),
        }
    }
}

#[napi]
pub struct Connection(endpoint::Connection);

#[napi]
impl Connection {
    #[napi]
    pub fn get_remote_node_id(&self) -> Result<PublicKey> {
        let id = endpoint::get_remote_node_id(&self.0)?;
        Ok(id.into())
    }

    #[napi]
    pub async fn open_uni(&self) -> Result<SendStream> {
        let s = self.0.open_uni().await.map_err(anyhow::Error::from)?;
        Ok(SendStream::new(s))
    }

    #[napi]
    pub async fn accept_uni(&self) -> Result<RecvStream> {
        let r = self.0.accept_uni().await.map_err(anyhow::Error::from)?;
        Ok(RecvStream::new(r))
    }

    #[napi]
    pub async fn open_bi(&self) -> Result<BiStream> {
        let (s, r) = self.0.open_bi().await.map_err(anyhow::Error::from)?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    #[napi]
    pub async fn accept_bi(&self) -> Result<BiStream> {
        let (s, r) = self.0.accept_bi().await.map_err(anyhow::Error::from)?;
        Ok(BiStream {
            send: SendStream::new(s),
            recv: RecvStream::new(r),
        })
    }

    #[napi]
    pub async fn read_datagram(&self) -> Result<Buffer> {
        let res = self.0.read_datagram().await.map_err(anyhow::Error::from)?;
        Ok(res.to_vec().into())
    }

    #[napi]
    pub async fn closed(&self) -> String {
        let err = self.0.closed().await;
        err.to_string()
    }

    #[napi]
    pub fn close_reason(&self) -> Option<String> {
        let err = self.0.close_reason();
        err.map(|s| s.to_string())
    }

    #[napi]
    pub fn close(&self, error_code: BigInt, reason: Uint8Array) -> Result<()> {
        let code =
            endpoint::VarInt::from_u64(error_code.get_u64().1).map_err(anyhow::Error::from)?;
        self.0.close(code, &reason);
        Ok(())
    }

    #[napi]
    pub fn send_datagram(&self, data: Uint8Array) -> Result<()> {
        self.0
            .send_datagram(data.to_vec().into())
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn send_datagram_wait(&self, data: Uint8Array) -> Result<()> {
        self.0
            .send_datagram_wait(data.to_vec().into())
            .await
            .map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub fn max_datagram_size(&self) -> Option<usize> {
        self.0.max_datagram_size()
    }

    #[napi]
    pub fn datagram_send_buffer_space(&self) -> usize {
        self.0.datagram_send_buffer_space()
    }

    #[napi]
    pub fn remote_address(&self) -> String {
        self.0.remote_address().to_string()
    }

    #[napi]
    pub fn local_ip(&self) -> Option<String> {
        self.0.local_ip().map(|s| s.to_string())
    }

    #[napi]
    pub fn rtt(&self) -> BigInt {
        self.0.rtt().as_millis().into()
    }

    #[napi]
    pub fn stable_id(&self) -> usize {
        self.0.stable_id()
    }

    #[napi]
    pub fn set_max_concurrent_uni_stream(&self, count: BigInt) -> Result<()> {
        let (_, n, _) = count.get_u64();
        let n = endpoint::VarInt::from_u64(n).map_err(anyhow::Error::from)?;
        self.0.set_max_concurrent_uni_streams(n);
        Ok(())
    }

    #[napi]
    pub fn set_receive_window(&self, count: BigInt) -> Result<()> {
        let (_, n, _) = count.get_u64();
        let n = endpoint::VarInt::from_u64(n).map_err(anyhow::Error::from)?;
        self.0.set_receive_window(n);
        Ok(())
    }

    #[napi]
    pub fn set_max_concurrent_bii_stream(&self, count: BigInt) -> Result<()> {
        let (_, n, _) = count.get_u64();
        let n = endpoint::VarInt::from_u64(n).map_err(anyhow::Error::from)?;
        self.0.set_max_concurrent_bi_streams(n);
        Ok(())
    }
}

#[napi]
pub struct BiStream {
    send: SendStream,
    recv: RecvStream,
}

#[napi]
impl BiStream {
    #[napi(getter)]
    pub fn send(&self) -> SendStream {
        self.send.clone()
    }

    #[napi(getter)]
    pub fn recv(&self) -> RecvStream {
        self.recv.clone()
    }
}

#[derive(Clone)]
#[napi]
pub struct SendStream(Arc<Mutex<endpoint::SendStream>>);

#[napi]
impl SendStream {
    fn new(s: endpoint::SendStream) -> Self {
        SendStream(Arc::new(Mutex::new(s)))
    }

    #[napi]
    pub async fn write(&self, buf: Uint8Array) -> Result<usize> {
        let mut s = self.0.lock().await;
        let written = s.write(&buf).await.map_err(anyhow::Error::from)?;
        Ok(written)
    }

    #[napi]
    pub async fn write_all(&self, buf: Uint8Array) -> Result<()> {
        let mut s = self.0.lock().await;
        s.write_all(&buf).await.map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn finish(&self) -> Result<()> {
        let mut s = self.0.lock().await;
        s.finish().map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn reset(&self, error_code: BigInt) -> Result<()> {
        let (_, n, _) = error_code.get_u64();
        let error_code = endpoint::VarInt::from_u64(n).map_err(anyhow::Error::from)?;
        let mut s = self.0.lock().await;
        s.reset(error_code).map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn set_priority(&self, p: i32) -> Result<()> {
        let s = self.0.lock().await;
        s.set_priority(p).map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn priority(&self) -> Result<i32> {
        let s = self.0.lock().await;
        let p = s.priority().map_err(anyhow::Error::from)?;
        Ok(p)
    }

    #[napi]
    pub async fn stopped(&self) -> Result<Option<BigInt>> {
        let mut s = self.0.lock().await;
        let res = s.stopped().await.map_err(anyhow::Error::from)?;
        let res = res.map(|r| r.into_inner().into());
        Ok(res)
    }

    #[napi]
    pub async fn id(&self) -> String {
        let r = self.0.lock().await;
        r.id().to_string()
    }
}

#[derive(Clone)]
#[napi]
pub struct RecvStream(Arc<Mutex<endpoint::RecvStream>>);

#[napi]
impl RecvStream {
    fn new(r: endpoint::RecvStream) -> Self {
        RecvStream(Arc::new(Mutex::new(r)))
    }

    #[napi]
    pub async fn read(&self, mut buf: Uint8Array) -> Result<Option<usize>> {
        let mut r = self.0.lock().await;
        let res = r.read(&mut buf).await.map_err(anyhow::Error::from)?;
        Ok(res)
    }

    #[napi]
    pub async fn read_exact(&self, mut buf: Uint8Array) -> Result<()> {
        let mut r = self.0.lock().await;
        r.read_exact(&mut buf).await.map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn read_to_end(&self, size_limit: u32) -> Result<Buffer> {
        let mut r = self.0.lock().await;
        let res = r
            .read_to_end(size_limit as _)
            .await
            .map_err(anyhow::Error::from)?;
        Ok(res.into())
    }

    #[napi]
    pub async fn id(&self) -> String {
        let r = self.0.lock().await;
        r.id().to_string()
    }

    #[napi]
    pub async fn stop(&self, error_code: BigInt) -> Result<()> {
        let (_, n, _) = error_code.get_u64();
        let error_code = endpoint::VarInt::from_u64(n).map_err(anyhow::Error::from)?;
        let mut r = self.0.lock().await;
        r.stop(error_code).map_err(anyhow::Error::from)?;
        Ok(())
    }

    #[napi]
    pub async fn received_reset(&self) -> Result<Option<BigInt>> {
        let mut r = self.0.lock().await;
        let code = r.received_reset().await.map_err(anyhow::Error::from)?;
        let code = code.map(|c| c.into_inner().into());
        Ok(code)
    }
}
