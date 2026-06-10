//! Multipath snapshots and event types.
//!
//! Mirrors the path-watcher surface from `iroh::endpoint::Connection::paths()`
//! and its associated streams. Paths are exposed as owned snapshots so they can
//! cross the FFI boundary without lifetime issues.

use std::sync::Arc;

use iroh::endpoint::LocalTransportAddr;
use iroh_base::TransportAddr;
use n0_future::{StreamExt, task::AbortOnDropHandle};

use crate::{CallbackError, WatchHandle};

/// A flat snapshot of an open path's state.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PathSnapshot {
    /// Opaque path identifier rendered as a string (upstream `PathId` is a u32
    /// wrapper but exposes no public accessor).
    pub id: String,
    /// True if this path is currently selected for application data.
    pub is_selected: bool,
    /// The remote transport address as a string. For IP paths this is
    /// `ip:port`; for relay paths this is the relay URL.
    pub remote_addr: String,
    /// True if this is a direct IP path.
    pub is_ip: bool,
    /// True if this is a relay path.
    pub is_relay: bool,
    /// RTT estimate in milliseconds (sampled from the live QUIC state).
    pub rtt_ms: u64,
    /// Flat headline statistics for this path.
    pub stats: PathStatsRecord,
}

/// Flattened headline numbers from `noq::PathStats`.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PathStatsRecord {
    /// RTT estimate (ms).
    pub rtt_ms: u64,
    /// UDP datagrams sent on this path.
    pub udp_tx_datagrams: u64,
    /// UDP bytes sent on this path.
    pub udp_tx_bytes: u64,
    /// UDP datagrams received on this path.
    pub udp_rx_datagrams: u64,
    /// UDP bytes received on this path.
    pub udp_rx_bytes: u64,
    /// Current congestion window.
    pub cwnd: u64,
    /// Congestion events on this path.
    pub congestion_events: u64,
    /// Packets considered lost on this path.
    pub lost_packets: u64,
    /// Bytes considered lost on this path.
    pub lost_bytes: u64,
    /// Largest UDP payload this path currently supports.
    pub current_mtu: u32,
}

impl From<iroh::endpoint::PathStats> for PathStatsRecord {
    fn from(s: iroh::endpoint::PathStats) -> Self {
        Self {
            rtt_ms: s.rtt.as_millis() as u64,
            udp_tx_datagrams: s.udp_tx.datagrams,
            udp_tx_bytes: s.udp_tx.bytes,
            udp_rx_datagrams: s.udp_rx.datagrams,
            udp_rx_bytes: s.udp_rx.bytes,
            cwnd: s.cwnd,
            congestion_events: s.congestion_events,
            lost_packets: s.lost_packets,
            lost_bytes: s.lost_bytes,
            current_mtu: s.current_mtu as u32,
        }
    }
}

fn transport_addr_to_string(addr: &TransportAddr) -> String {
    match addr {
        TransportAddr::Ip(socket) => socket.to_string(),
        TransportAddr::Relay(url) => url.to_string(),
        _ => "unknown".to_string(),
    }
}

fn local_transport_addr_to_string(addr: &LocalTransportAddr) -> String {
    match addr {
        LocalTransportAddr::Ip(Some(ip)) => ip.to_string(),
        LocalTransportAddr::Ip(None) => "unknown".to_string(),
        LocalTransportAddr::Relay(url) => url.to_string(),
        LocalTransportAddr::Custom(Some(c)) => format!("{c:?}"),
        _ => "unknown".to_string(),
    }
}

/// An event from `Connection::path_events`.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum PathEvent {
    /// A new network path was opened.
    Opened {
        id: String,
        remote_addr: String,
        local_addr: String,
    },
    /// A network path was closed.
    Closed {
        id: String,
        remote_addr: String,
        local_addr: String,
        last_stats: PathStatsRecord,
    },
    /// This path was selected for transmission of application data.
    Selected {
        id: String,
        remote_addr: String,
        local_addr: String,
    },
    /// Events were dropped before the subscriber received them.
    Lagged { missed: u64 },
}

impl From<iroh::endpoint::PathEvent> for PathEvent {
    fn from(e: iroh::endpoint::PathEvent) -> Self {
        match e {
            iroh::endpoint::PathEvent::Opened {
                id,
                remote_addr,
                local_addr,
                ..
            } => Self::Opened {
                id: id.to_string(),
                remote_addr: transport_addr_to_string(&remote_addr),
                local_addr: local_transport_addr_to_string(&local_addr),
            },
            iroh::endpoint::PathEvent::Closed {
                id,
                remote_addr,
                local_addr,
                last_stats,
                ..
            } => Self::Closed {
                id: id.to_string(),
                remote_addr: transport_addr_to_string(&remote_addr),
                local_addr: local_transport_addr_to_string(&local_addr),
                last_stats: (*last_stats).into(),
            },
            iroh::endpoint::PathEvent::Selected {
                id,
                remote_addr,
                local_addr,
                ..
            } => Self::Selected {
                id: id.to_string(),
                remote_addr: transport_addr_to_string(&remote_addr),
                local_addr: local_transport_addr_to_string(&local_addr),
            },
            iroh::endpoint::PathEvent::Lagged { missed, .. } => Self::Lagged { missed },
            _ => Self::Lagged { missed: 0 },
        }
    }
}

/// Callback for `Connection::watch_paths` — fires whenever the open-paths
/// snapshot changes (path opens/closes/selection changes).
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait PathChangeCallback: Send + Sync + 'static {
    async fn on_change(&self, paths: Vec<PathSnapshot>) -> Result<(), CallbackError>;
}

/// Callback for `Connection::watch_path_events` — fires for each individual
/// path event.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait PathEventCallback: Send + Sync + 'static {
    async fn on_event(&self, event: PathEvent) -> Result<(), CallbackError>;
}

pub(crate) fn snapshot_paths(conn: &iroh::endpoint::Connection) -> Vec<PathSnapshot> {
    conn.paths()
        .iter()
        .map(|p| {
            let stats = p.stats();
            PathSnapshot {
                id: p.id().to_string(),
                is_selected: p.is_selected(),
                remote_addr: transport_addr_to_string(p.remote_addr()),
                is_ip: p.is_ip(),
                is_relay: p.is_relay(),
                rtt_ms: p.rtt().as_millis() as u64,
                stats: stats.into(),
            }
        })
        .collect()
}

pub(crate) fn spawn_paths_watch(
    conn: iroh::endpoint::Connection,
    cb: Arc<dyn PathChangeCallback>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        let mut stream = conn.paths_stream();
        while let Some(snapshot) = stream.next().await {
            let mapped: Vec<PathSnapshot> = snapshot
                .iter()
                .map(|p| {
                    let stats = p.stats();
                    PathSnapshot {
                        id: p.id().to_string(),
                        is_selected: p.is_selected(),
                        remote_addr: transport_addr_to_string(p.remote_addr()),
                        is_ip: p.is_ip(),
                        is_relay: p.is_relay(),
                        rtt_ms: p.rtt().as_millis() as u64,
                        stats: stats.into(),
                    }
                })
                .collect();
            if let Err(err) = cb.on_change(mapped).await {
                tracing::warn!("paths watch callback error: {err:?}");
                break;
            }
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}

pub(crate) fn spawn_path_events_watch(
    conn: iroh::endpoint::Connection,
    cb: Arc<dyn PathEventCallback>,
) -> WatchHandle {
    let task = n0_future::task::spawn(async move {
        let mut stream = conn.path_events();
        while let Some(event) = stream.next().await {
            let mapped: PathEvent = event.into();
            if let Err(err) = cb.on_event(mapped).await {
                tracing::warn!("path events callback error: {err:?}");
                break;
            }
        }
    });
    WatchHandle::new(AbortOnDropHandle::new(task))
}
