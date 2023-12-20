//! The runtime module provides the iroh runtime, consisting of a general purpose
//! tokio runtime and a set of single threaded runtimes.
use std::sync::Arc;

/// A handle to the iroh runtime
#[derive(Debug, Clone)]
pub struct Handle {
    inner: Arc<HandleInner>,
}

impl Handle {
    /// Create a new iroh runtime consisting of a tokio runtime and a thread per
    /// core runtime.
    pub fn new(rt: tokio::runtime::Handle) -> Self {
        Self {
            inner: Arc::new(HandleInner { rt }),
        }
    }

    /// Get a handle to the main tokio runtime
    pub fn main(&self) -> &tokio::runtime::Handle {
        &self.inner.rt
    }
}

#[derive(Debug)]
struct HandleInner {
    rt: tokio::runtime::Handle,
}
