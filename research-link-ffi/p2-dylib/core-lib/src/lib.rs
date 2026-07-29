//! Stand-in for `iroh-ffi`: defines the shared FFI object type.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

uniffi::setup_scaffolding!("p2_core_lib");

/// A process-global, so we can prove at runtime whether two libraries share
/// one copy of this crate or each got their own.
static GLOBAL_INSTANCES: AtomicU64 = AtomicU64::new(0);

/// Stand-in for `iroh_ffi::Endpoint` — an opaque object holding a "real" Rust
/// value that a downstream protocol crate needs access to.
#[derive(uniffi::Object)]
pub struct Endpoint {
    inner: Inner,
}

/// Stand-in for `iroh::Endpoint` — a plain Rust type with no stable ABI.
#[derive(Debug, Clone)]
pub struct Inner {
    pub id: u64,
}

#[uniffi::export]
impl Endpoint {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        let id = GLOBAL_INSTANCES.fetch_add(1, Ordering::SeqCst);
        Arc::new(Self {
            inner: Inner { id },
        })
    }

    pub fn id(&self) -> u64 {
        self.inner.id
    }

    /// Address of the process-global counter. If two loaded libraries report
    /// different addresses, they each have their own static copy of this crate.
    pub fn global_addr(&self) -> u64 {
        &GLOBAL_INSTANCES as *const _ as u64
    }
}

impl Endpoint {
    /// The Rust-native accessor a protocol crate needs. Not part of the FFI.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}
