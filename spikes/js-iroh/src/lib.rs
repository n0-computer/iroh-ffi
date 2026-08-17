#![deny(clippy::all)]
use std::sync::Arc;

use napi_derive::napi;

/// napi wrapper over `iroh_ffi::Endpoint`. Lives in a Rust `dylib` so plugin addons can
/// name this exact type — that is what makes `napi_unwrap` across addons sound.
#[napi]
pub struct Endpoint {
    inner: Arc<iroh_ffi::Endpoint>,
}

#[napi]
impl Endpoint {
    #[napi(factory)]
    pub async fn bind() -> napi::Result<Endpoint> {
        let opts = iroh_ffi::EndpointOptions {
            preset: Some(iroh_ffi::preset_minimal()),
            ..Default::default()
        };
        let inner = iroh_ffi::Endpoint::bind(opts)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e.message()))?;
        Ok(Endpoint { inner: Arc::new(inner) })
    }

    #[napi]
    pub fn addr(&self) -> EndpointAddr {
        EndpointAddr {
            inner: self.inner.addr(),
        }
    }

    #[napi]
    pub async fn close(&self) -> napi::Result<()> {
        self.inner
            .close()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e.message()))?;
        Ok(())
    }
}

impl Endpoint {
    /// Escape hatch for plugin addons.
    pub fn raw(&self) -> &iroh::Endpoint {
        self.inner.raw()
    }
}

#[napi]
pub struct EndpointAddr {
    inner: Arc<iroh_ffi::EndpointAddr>,
}

#[napi]
impl EndpointAddr {
    #[napi]
    pub fn to_string_(&self) -> String {
        self.inner.to_string()
    }
}

impl EndpointAddr {
    pub fn raw(&self) -> &iroh_ffi::EndpointAddr {
        &self.inner
    }
}
