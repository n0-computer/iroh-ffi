//! Watcher callback bridges.
//!
//! `iroh::Endpoint` exposes a few values that change over time via the
//! `n0_watcher::Watcher` trait (`watch_addr`, `home_relay_status`, etc.). That
//! trait doesn't map naturally to uniffi, so the FFI exposes the same data via
//! callback traits: register a callback and get back a [`WatchHandle`] that
//! aborts the underlying task when dropped (or when [`WatchHandle::stop`] is
//! called).

use std::sync::Arc;

use iroh::Watcher;
use n0_future::{StreamExt, task::AbortOnDropHandle};
use tokio::sync::Mutex;

use crate::{CallbackError, EndpointAddr};

/// Callback invoked whenever the endpoint's [`EndpointAddr`] changes.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AddrChangeCallback: Send + Sync + 'static {
    async fn on_change(&self, addr: Arc<EndpointAddr>) -> Result<(), CallbackError>;
}

/// Callback invoked whenever the home-relay connection status list changes.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait HomeRelayCallback: Send + Sync + 'static {
    async fn on_change(&self, relay_urls: Vec<String>) -> Result<(), CallbackError>;
}

/// Callback invoked when a network-stack change is detected (interface up/down,
/// roaming, etc.).
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait NetworkChangeCallback: Send + Sync + 'static {
    async fn on_change(&self) -> Result<(), CallbackError>;
}

/// Handle to a running watcher task. Drop it (or call [`Self::stop`]) to
/// unregister the callback.
#[derive(uniffi::Object)]
pub struct WatchHandle {
    task: Mutex<Option<AbortOnDropHandle<()>>>,
}

impl WatchHandle {
    pub(crate) fn new(task: AbortOnDropHandle<()>) -> Self {
        Self {
            task: Mutex::new(Some(task)),
        }
    }
}

#[uniffi::export]
impl WatchHandle {
    /// Stop the watcher, aborting the background task.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn stop(&self) {
        self.task.lock().await.take();
    }
}

pub(crate) fn spawn_watch_addr(
    endpoint: iroh::Endpoint,
    cb: Arc<dyn AddrChangeCallback>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        let mut stream = endpoint.watch_addr().stream();
        while let Some(addr) = stream.next().await {
            let mapped: EndpointAddr = addr.into();
            if let Err(err) = cb.on_change(Arc::new(mapped)).await {
                tracing::warn!("addr change callback error: {err:?}");
                break;
            }
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_home_relay_watch(
    endpoint: iroh::Endpoint,
    cb: Arc<dyn HomeRelayCallback>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        let mut stream = endpoint.home_relay_status().stream();
        while let Some(statuses) = stream.next().await {
            let urls: Vec<String> = statuses.into_iter().map(|s| s.url().to_string()).collect();
            if let Err(err) = cb.on_change(urls).await {
                tracing::warn!("home relay callback error: {err:?}");
                break;
            }
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_network_change_watch(
    endpoint: iroh::Endpoint,
    cb: Arc<dyn NetworkChangeCallback>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        loop {
            endpoint.network_change().await;
            if let Err(err) = cb.on_change().await {
                tracing::warn!("network change callback error: {err:?}");
                break;
            }
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}
