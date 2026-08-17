#![deny(clippy::all)]
use napi_derive::napi;

/// Ping protocol, in a SEPARATE addon from the Endpoint it operates on.
#[napi]
pub struct Ping {
    inner: iroh_ping::Ping,
    router: std::sync::Mutex<Option<iroh::protocol::Router>>,
}

#[napi]
impl Ping {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: iroh_ping::Ping::new(),
            router: std::sync::Mutex::new(None),
        }
    }

    /// Accept ping on `endpoint`. The plugin owns its own router, reached through the
    /// `raw()` escape hatch on the shared Endpoint type.
    ///
    /// `async` is load-bearing, not cosmetic: `Router::spawn()` needs a tokio reactor, and a
    /// sync `#[napi]` method runs on the JS thread with no tokio context — it panics with
    /// "there is no reactor running". Same trap as iroh-ffi's sync `watch_*` methods, which
    /// capture a `tokio::runtime::Handle` at construction instead.
    #[napi]
    pub async fn serve(&self, endpoint: &js_iroh::Endpoint) -> napi::Result<()> {
        let router = iroh::protocol::Router::builder(endpoint.raw().clone())
            .accept(iroh_ping::ALPN, self.inner.clone())
            .spawn();
        *self.router.lock().unwrap() = Some(router);
        Ok(())
    }

    /// THE LOAD-BEARING CALL: takes an `Endpoint` created by the core addon.
    #[napi]
    pub async fn ping(
        &self,
        endpoint: &js_iroh::Endpoint,
        addr: &js_iroh::EndpointAddr,
    ) -> napi::Result<u32> {
        let iroh_addr: iroh::EndpointAddr = addr
            .raw()
            .clone()
            .try_into()
            .map_err(|e: iroh_ffi::IrohError| anyhow::anyhow!("{}", e.message()))?;
        let rtt = self
            .inner
            .ping(endpoint.raw(), iroh_addr)
            .await
            .map_err(|e| anyhow::anyhow!("{e:#}"))?;
        Ok(rtt.as_millis() as u32)
    }
}

#[napi]
pub fn alpn() -> Vec<u8> {
    iroh_ping::ALPN.to_vec()
}
