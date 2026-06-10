use n0_future::{StreamExt, task::AbortOnDropHandle};
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use tokio::sync::Mutex;

use crate::{EndpointAddr, PathEvent, PathSnapshot};

/// Handle to a running watcher task. Call `stop()` (or drop) to unregister.
#[napi]
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

#[napi]
impl WatchHandle {
    /// Stop the watcher, aborting the background task.
    #[napi]
    pub async fn stop(&self) {
        self.task.lock().await.take();
    }
}

pub(crate) fn spawn_watch_addr(
    endpoint: iroh::Endpoint,
    cb: ThreadsafeFunction<EndpointAddr>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        use iroh::Watcher;
        let mut stream = endpoint.watch_addr().stream();
        while let Some(addr) = stream.next().await {
            let mapped: EndpointAddr = addr.into();
            cb.call(Ok(mapped), ThreadsafeFunctionCallMode::NonBlocking);
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_home_relay_watch(
    endpoint: iroh::Endpoint,
    cb: ThreadsafeFunction<Vec<String>>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        use iroh::Watcher;
        let mut stream = endpoint.home_relay_status().stream();
        while let Some(statuses) = stream.next().await {
            let urls: Vec<String> = statuses.into_iter().map(|s| s.url().to_string()).collect();
            cb.call(Ok(urls), ThreadsafeFunctionCallMode::NonBlocking);
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_network_change_watch(
    endpoint: iroh::Endpoint,
    cb: ThreadsafeFunction<()>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        loop {
            endpoint.network_change().await;
            cb.call(Ok(()), ThreadsafeFunctionCallMode::NonBlocking);
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_paths_watch(
    conn: iroh::endpoint::Connection,
    cb: ThreadsafeFunction<Vec<PathSnapshot>>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        let mut stream = conn.paths_stream();
        while let Some(snapshot) = stream.next().await {
            let mapped: Vec<PathSnapshot> = snapshot
                .iter()
                .map(|p| PathSnapshot {
                    id: p.id().to_string(),
                    is_selected: p.is_selected(),
                    remote_addr: crate::path::transport_addr_to_string(p.remote_addr()),
                    is_ip: p.is_ip(),
                    is_relay: p.is_relay(),
                    rtt_ms: p.rtt().as_millis() as i64,
                    stats: p.stats().into(),
                })
                .collect();
            cb.call(Ok(mapped), ThreadsafeFunctionCallMode::NonBlocking);
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_path_events_watch(
    conn: iroh::endpoint::Connection,
    cb: ThreadsafeFunction<PathEvent>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        let mut stream = conn.path_events();
        while let Some(event) = stream.next().await {
            let mapped: PathEvent = event.into();
            cb.call(Ok(mapped), ThreadsafeFunctionCallMode::NonBlocking);
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}
