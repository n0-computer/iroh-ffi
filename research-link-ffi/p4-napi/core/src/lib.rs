use std::sync::atomic::{AtomicU64, Ordering};

use napi_derive::napi;

static GLOBAL_INSTANCES: AtomicU64 = AtomicU64::new(0);

/// Stand-in for `iroh_js::Endpoint`.
#[napi]
pub struct Endpoint {
    inner: Inner,
}

#[derive(Debug, Clone)]
pub struct Inner {
    pub id: u64,
}

#[napi]
impl Endpoint {
    #[napi(constructor)]
    pub fn new() -> Self {
        let id = GLOBAL_INSTANCES.fetch_add(1, Ordering::SeqCst);
        Self {
            inner: Inner { id },
        }
    }

    #[napi]
    pub fn id(&self) -> u32 {
        self.inner.id as u32
    }

    #[napi]
    pub fn global_addr(&self) -> String {
        format!("{:#x}", &GLOBAL_INSTANCES as *const _ as usize)
    }
}

impl Endpoint {
    /// The Rust-native accessor a downstream protocol addon needs.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}
