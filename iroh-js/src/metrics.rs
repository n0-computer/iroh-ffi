use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
/// Metrics collected by an [`crate::endpoint::Endpoint`].
pub struct EndpointMetrics {
    /// Metrics collected by the endpoint's socket.
    pub magicsock: MagicsockMetrics,
    /// Metrics collected by net reports.
    pub net_report: NetReportMetrics,
    /// Metrics collected by the portmapper service.
    pub portmapper: PortmapMetrics,
}

#[napi(object)]
pub struct MagicsockMetrics {
    pub re_stun_calls: BigInt,
    pub update_direct_addrs: BigInt,

    // Sends (data or disco)
    pub send_ipv4: BigInt,
    pub send_ipv6: BigInt,
    pub send_relay: BigInt,
    pub send_relay_error: BigInt,

    // Data packets (non-disco)
    pub send_data: BigInt,
    pub send_data_network_down: BigInt,
    pub recv_data_relay: BigInt,
    pub recv_data_ipv4: BigInt,
    pub recv_data_ipv6: BigInt,
    /// Number of QUIC datagrams received.
    pub recv_datagrams: BigInt,
    /// Number of datagrams received using GRO
    pub recv_gro_datagrams: BigInt,

    // Disco packets
    pub send_disco_udp: BigInt,
    pub send_disco_relay: BigInt,
    pub sent_disco_udp: BigInt,
    pub sent_disco_relay: BigInt,
    pub sent_disco_ping: BigInt,
    pub sent_disco_pong: BigInt,
    pub sent_disco_call_me_maybe: BigInt,
    pub recv_disco_bad_key: BigInt,
    pub recv_disco_bad_parse: BigInt,

    pub recv_disco_udp: BigInt,
    pub recv_disco_relay: BigInt,
    pub recv_disco_ping: BigInt,
    pub recv_disco_pong: BigInt,
    pub recv_disco_call_me_maybe: BigInt,
    pub recv_disco_call_me_maybe_bad_disco: BigInt,

    // How many times our relay home node DI has changed from non-zero to a different non-zero.
    pub relay_home_change: BigInt,

    /*
     * Connection Metrics
     */
    /// The number of direct connections we have made to peers.
    pub num_direct_conns_added: BigInt,
    /// The number of direct connections we have lost to peers.
    pub num_direct_conns_removed: BigInt,
    /// The number of connections to peers we have added over relay.
    pub num_relay_conns_added: BigInt,
    /// The number of connections to peers we have removed over relay.
    pub num_relay_conns_removed: BigInt,

    pub actor_tick_main: BigInt,
    pub actor_tick_msg: BigInt,
    pub actor_tick_re_stun: BigInt,
    pub actor_tick_portmap_changed: BigInt,
    pub actor_tick_direct_addr_heartbeat: BigInt,
    pub actor_tick_direct_addr_update_receiver: BigInt,
    pub actor_link_change: BigInt,
    pub actor_tick_other: BigInt,

    /// Number of nodes we have attempted to contact.
    pub nodes_contacted: BigInt,
    /// Number of nodes we have managed to contact directly.
    pub nodes_contacted_directly: BigInt,

    /// Number of connections with a successful handshake.
    pub connection_handshake_success: BigInt,
    /// Number of connections with a successful handshake that became direct.
    pub connection_became_direct: BigInt,
}

impl From<Arc<iroh::metrics::MagicsockMetrics>> for MagicsockMetrics {
    fn from(value: Arc<iroh::metrics::MagicsockMetrics>) -> Self {
        Self {
            re_stun_calls: value.re_stun_calls.get().into(),
            update_direct_addrs: value.update_direct_addrs.get().into(),
            send_ipv4: value.send_ipv4.get().into(),
            send_ipv6: value.send_ipv6.get().into(),
            send_relay: value.send_relay.get().into(),
            send_relay_error: value.send_relay_error.get().into(),
            send_data: value.send_data.get().into(),
            send_data_network_down: value.send_data_network_down.get().into(),
            recv_data_relay: value.recv_data_relay.get().into(),
            recv_data_ipv4: value.recv_data_ipv4.get().into(),
            recv_data_ipv6: value.recv_data_ipv6.get().into(),
            recv_datagrams: value.recv_datagrams.get().into(),
            recv_gro_datagrams: value.recv_gro_datagrams.get().into(),
            send_disco_udp: value.send_disco_udp.get().into(),
            send_disco_relay: value.send_disco_relay.get().into(),
            sent_disco_udp: value.sent_disco_udp.get().into(),
            sent_disco_relay: value.sent_disco_relay.get().into(),
            sent_disco_ping: value.sent_disco_ping.get().into(),
            sent_disco_pong: value.sent_disco_pong.get().into(),
            sent_disco_call_me_maybe: value.sent_disco_call_me_maybe.get().into(),
            recv_disco_bad_key: value.recv_disco_bad_key.get().into(),
            recv_disco_bad_parse: value.recv_disco_bad_parse.get().into(),
            recv_disco_udp: value.recv_disco_udp.get().into(),
            recv_disco_relay: value.recv_disco_relay.get().into(),
            recv_disco_ping: value.recv_disco_ping.get().into(),
            recv_disco_pong: value.recv_disco_pong.get().into(),
            recv_disco_call_me_maybe: value.recv_disco_call_me_maybe.get().into(),
            recv_disco_call_me_maybe_bad_disco: value
                .recv_disco_call_me_maybe_bad_disco
                .get()
                .into(),
            relay_home_change: value.relay_home_change.get().into(),
            num_direct_conns_added: value.num_direct_conns_added.get().into(),
            num_direct_conns_removed: value.num_direct_conns_removed.get().into(),
            num_relay_conns_added: value.num_relay_conns_added.get().into(),
            num_relay_conns_removed: value.num_relay_conns_removed.get().into(),
            actor_tick_main: value.actor_tick_main.get().into(),
            actor_tick_msg: value.actor_tick_msg.get().into(),
            actor_tick_re_stun: value.actor_tick_re_stun.get().into(),
            actor_tick_portmap_changed: value.actor_tick_portmap_changed.get().into(),
            actor_tick_direct_addr_heartbeat: value.actor_tick_direct_addr_heartbeat.get().into(),
            actor_tick_direct_addr_update_receiver: value
                .actor_tick_direct_addr_update_receiver
                .get()
                .into(),
            actor_link_change: value.actor_link_change.get().into(),
            actor_tick_other: value.actor_tick_other.get().into(),
            nodes_contacted: value.nodes_contacted.get().into(),
            nodes_contacted_directly: value.nodes_contacted_directly.get().into(),
            connection_handshake_success: value.connection_handshake_success.get().into(),
            connection_became_direct: value.connection_became_direct.get().into(),
        }
    }
}

#[derive(Clone)]
#[napi(object)]
/// Metrics collected by net reports.
pub struct NetReportMetrics {
    /// Incoming STUN packets dropped due to a full receiving queue.
    pub stun_packets_dropped: BigInt,
    /// Number of IPv4 STUN packets sent.
    pub stun_packets_sent_ipv4: BigInt,
    /// Number of IPv6 STUN packets sent.
    pub stun_packets_sent_ipv6: BigInt,
    /// Number of IPv4 STUN packets received.
    pub stun_packets_recv_ipv4: BigInt,
    /// Number of IPv6 STUN packets received.
    pub stun_packets_recv_ipv6: BigInt,
    /// Number of reports executed by net_report, including full reports.
    pub reports: BigInt,
    /// Number of full reports executed by net_report
    pub reports_full: BigInt,
}

impl From<Arc<iroh::metrics::NetReportMetrics>> for NetReportMetrics {
    fn from(value: Arc<iroh::metrics::NetReportMetrics>) -> Self {
        Self {
            stun_packets_dropped: value.stun_packets_dropped.get().into(),
            stun_packets_sent_ipv4: value.stun_packets_sent_ipv4.get().into(),
            stun_packets_sent_ipv6: value.stun_packets_sent_ipv6.get().into(),
            stun_packets_recv_ipv4: value.stun_packets_recv_ipv4.get().into(),
            stun_packets_recv_ipv6: value.stun_packets_recv_ipv6.get().into(),
            reports: value.reports.get().into(),
            reports_full: value.reports_full.get().into(),
        }
    }
}

#[derive(Clone)]
#[napi(object)]
/// Metrics collected by the portmapper service.
pub struct PortmapMetrics {
    /*
     * General port mapping metrics
     */
    /// Number of probing tasks started.
    pub probes_started: BigInt,
    /// Number of updates to the local port.
    pub local_port_updates: BigInt,
    /// Number of mapping tasks started.
    pub mapping_attempts: BigInt,
    /// Number of failed mapping tasks.
    pub mapping_failures: BigInt,
    /// Number of times the external address obtained via port mapping was updated.
    pub external_address_updated: BigInt,

    /*
     * UPnP metrics
     */
    /// Number of UPnP probes executed.
    pub upnp_probes: BigInt,
    /// Number of failed Upnp probes.
    pub upnp_probes_failed: BigInt,
    /// Number of UPnP probes that found it available.
    pub upnp_available: BigInt,
    /// Number of UPnP probes that resulted in a gateway different to the previous one,
    pub upnp_gateway_updated: BigInt,

    /*
     * PCP metrics
     */
    /// Number of PCP probes executed.
    pub pcp_probes: BigInt,
    /// Number of PCP probes that found it available.
    pub pcp_available: BigInt,
}

impl From<Arc<iroh::metrics::PortmapMetrics>> for PortmapMetrics {
    fn from(value: Arc<iroh::metrics::PortmapMetrics>) -> Self {
        Self {
            probes_started: value.probes_started.get().into(),
            local_port_updates: value.local_port_updates.get().into(),
            mapping_attempts: value.mapping_attempts.get().into(),
            mapping_failures: value.mapping_failures.get().into(),
            external_address_updated: value.external_address_updated.get().into(),
            upnp_probes: value.upnp_probes.get().into(),
            upnp_probes_failed: value.upnp_probes_failed.get().into(),
            upnp_available: value.upnp_available.get().into(),
            upnp_gateway_updated: value.upnp_gateway_updated.get().into(),
            pcp_probes: value.pcp_probes.get().into(),
            pcp_available: value.pcp_available.get().into(),
        }
    }
}
