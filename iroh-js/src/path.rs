use iroh::endpoint::LocalTransportAddr;
use iroh_base::TransportAddr;
use napi_derive::napi;

/// Flattened headline numbers from `noq::PathStats`.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct PathStatsRecord {
    pub rtt_ms: i64,
    pub udp_tx_datagrams: i64,
    pub udp_tx_bytes: i64,
    pub udp_rx_datagrams: i64,
    pub udp_rx_bytes: i64,
    pub cwnd: i64,
    pub congestion_events: i64,
    pub lost_packets: i64,
    pub lost_bytes: i64,
    pub current_mtu: u32,
}

impl From<iroh::endpoint::PathStats> for PathStatsRecord {
    fn from(s: iroh::endpoint::PathStats) -> Self {
        Self {
            rtt_ms: s.rtt.as_millis() as i64,
            udp_tx_datagrams: s.udp_tx.datagrams as i64,
            udp_tx_bytes: s.udp_tx.bytes as i64,
            udp_rx_datagrams: s.udp_rx.datagrams as i64,
            udp_rx_bytes: s.udp_rx.bytes as i64,
            cwnd: s.cwnd as i64,
            congestion_events: s.congestion_events as i64,
            lost_packets: s.lost_packets as i64,
            lost_bytes: s.lost_bytes as i64,
            current_mtu: s.current_mtu as u32,
        }
    }
}

/// A flat snapshot of an open path's state.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct PathSnapshot {
    pub id: String,
    pub is_selected: bool,
    pub remote_addr: String,
    pub is_ip: bool,
    pub is_relay: bool,
    pub rtt_ms: i64,
    pub stats: PathStatsRecord,
}

pub(crate) fn transport_addr_to_string(addr: &TransportAddr) -> String {
    match addr {
        TransportAddr::Ip(socket) => socket.to_string(),
        TransportAddr::Relay(url) => url.to_string(),
        _ => "unknown".to_string(),
    }
}

pub(crate) fn local_transport_addr_to_string(addr: &LocalTransportAddr) -> String {
    match addr {
        LocalTransportAddr::Ip(Some(ip)) => ip.to_string(),
        LocalTransportAddr::Ip(None) => "unknown".to_string(),
        LocalTransportAddr::Relay(url) => url.to_string(),
        LocalTransportAddr::Custom(Some(c)) => format!("{c:?}"),
        _ => "unknown".to_string(),
    }
}

/// An event from `Connection::watchPathEvents`.
#[derive(Debug, Clone)]
#[napi(string_enum)]
pub enum PathEventKind {
    Opened,
    Closed,
    Selected,
    Lagged,
}

/// A path event with its associated data.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct PathEvent {
    pub kind: PathEventKind,
    pub id: Option<String>,
    pub remote_addr: Option<String>,
    pub local_addr: Option<String>,
    pub last_stats: Option<PathStatsRecord>,
    pub missed: Option<i64>,
}

impl From<iroh::endpoint::PathEvent> for PathEvent {
    fn from(e: iroh::endpoint::PathEvent) -> Self {
        match e {
            iroh::endpoint::PathEvent::Opened {
                id,
                remote_addr,
                local_addr,
                ..
            } => PathEvent {
                kind: PathEventKind::Opened,
                id: Some(id.to_string()),
                remote_addr: Some(transport_addr_to_string(&remote_addr)),
                local_addr: Some(local_transport_addr_to_string(&local_addr)),
                last_stats: None,
                missed: None,
            },
            iroh::endpoint::PathEvent::Closed {
                id,
                remote_addr,
                local_addr,
                last_stats,
                ..
            } => PathEvent {
                kind: PathEventKind::Closed,
                id: Some(id.to_string()),
                remote_addr: Some(transport_addr_to_string(&remote_addr)),
                local_addr: Some(local_transport_addr_to_string(&local_addr)),
                last_stats: Some((*last_stats).into()),
                missed: None,
            },
            iroh::endpoint::PathEvent::Selected {
                id,
                remote_addr,
                local_addr,
                ..
            } => PathEvent {
                kind: PathEventKind::Selected,
                id: Some(id.to_string()),
                remote_addr: Some(transport_addr_to_string(&remote_addr)),
                local_addr: Some(local_transport_addr_to_string(&local_addr)),
                last_stats: None,
                missed: None,
            },
            iroh::endpoint::PathEvent::Lagged { missed, .. } => PathEvent {
                kind: PathEventKind::Lagged,
                id: None,
                remote_addr: None,
                local_addr: None,
                last_stats: None,
                missed: Some(missed as i64),
            },
            _ => PathEvent {
                kind: PathEventKind::Lagged,
                id: None,
                remote_addr: None,
                local_addr: None,
                last_stats: None,
                missed: Some(0),
            },
        }
    }
}

pub(crate) fn snapshot_paths(conn: &iroh::endpoint::Connection) -> Vec<PathSnapshot> {
    conn.paths()
        .iter()
        .map(|p| PathSnapshot {
            id: p.id().to_string(),
            is_selected: p.is_selected(),
            remote_addr: transport_addr_to_string(p.remote_addr()),
            is_ip: p.is_ip(),
            is_relay: p.is_relay(),
            rtt_ms: p.rtt().as_millis() as i64,
            stats: p.stats().into(),
        })
        .collect()
}
