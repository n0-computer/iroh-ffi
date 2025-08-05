use std::sync::Arc;

#[derive(uniffi::Object)]
/// Metrics collected by an [`crate::endpoint::Endpoint`].
pub struct EndpointMetrics {
    /// Metrics collected by the endpoint's socket.
    pub magicsock: MagicsockMetrics,
    /// Metrics collected by net reports.
    pub net_report: NetReportMetrics,
    /// Metrics collected by the portmapper service.
    pub portmapper: PortmapMetrics,
}

#[derive(uniffi::Object)]
pub struct MagicsockMetrics {
    pub re_stun_calls: u64,
    pub update_direct_addrs: u64,

    // Sends (data or disco)
    pub send_ipv4: u64,
    pub send_ipv6: u64,
    pub send_relay: u64,
    pub send_relay_error: u64,

    // Data packets (non-disco)
    pub send_data: u64,
    pub send_data_network_down: u64,
    pub recv_data_relay: u64,
    pub recv_data_ipv4: u64,
    pub recv_data_ipv6: u64,
    /// Number of QUIC datagrams received.
    pub recv_datagrams: u64,
    /// Number of datagrams received using GRO
    pub recv_gro_datagrams: u64,

    // Disco packets
    pub send_disco_udp: u64,
    pub send_disco_relay: u64,
    pub sent_disco_udp: u64,
    pub sent_disco_relay: u64,
    pub sent_disco_ping: u64,
    pub sent_disco_pong: u64,
    pub sent_disco_call_me_maybe: u64,
    pub recv_disco_bad_key: u64,
    pub recv_disco_bad_parse: u64,

    pub recv_disco_udp: u64,
    pub recv_disco_relay: u64,
    pub recv_disco_ping: u64,
    pub recv_disco_pong: u64,
    pub recv_disco_call_me_maybe: u64,
    pub recv_disco_call_me_maybe_bad_disco: u64,

    // How many times our relay home node DI has changed from non-zero to a different non-zero.
    pub relay_home_change: u64,

    /*
     * Connection Metrics
     */
    /// The number of direct connections we have made to peers.
    pub num_direct_conns_added: u64,
    /// The number of direct connections we have lost to peers.
    pub num_direct_conns_removed: u64,
    /// The number of connections to peers we have added over relay.
    pub num_relay_conns_added: u64,
    /// The number of connections to peers we have removed over relay.
    pub num_relay_conns_removed: u64,

    pub actor_tick_main: u64,
    pub actor_tick_msg: u64,
    pub actor_tick_re_stun: u64,
    pub actor_tick_portmap_changed: u64,
    pub actor_tick_direct_addr_heartbeat: u64,
    pub actor_tick_direct_addr_update_receiver: u64,
    pub actor_link_change: u64,
    pub actor_tick_other: u64,

    /// Number of nodes we have attempted to contact.
    pub nodes_contacted: u64,
    /// Number of nodes we have managed to contact directly.
    pub nodes_contacted_directly: u64,

    /// Number of connections with a successful handshake.
    pub connection_handshake_success: u64,
    /// Number of connections with a successful handshake that became direct.
    pub connection_became_direct: u64,
}

impl From<Arc<iroh::metrics::MagicsockMetrics>> for MagicsockMetrics {
    fn from(value: Arc<iroh::metrics::MagicsockMetrics>) -> Self {
        Self {
            re_stun_calls: value.re_stun_calls.get(),
            update_direct_addrs: value.update_direct_addrs.get(),
            send_ipv4: value.send_ipv4.get(),
            send_ipv6: value.send_ipv6.get(),
            send_relay: value.send_relay.get(),
            send_relay_error: value.send_relay_error.get(),
            send_data: value.send_data.get(),
            send_data_network_down: value.send_data_network_down.get(),
            recv_data_relay: value.recv_data_relay.get(),
            recv_data_ipv4: value.recv_data_ipv4.get(),
            recv_data_ipv6: value.recv_data_ipv6.get(),
            recv_datagrams: value.recv_datagrams.get(),
            recv_gro_datagrams: value.recv_gro_datagrams.get(),
            send_disco_udp: value.send_disco_udp.get(),
            send_disco_relay: value.send_disco_relay.get(),
            sent_disco_udp: value.sent_disco_udp.get(),
            sent_disco_relay: value.sent_disco_relay.get(),
            sent_disco_ping: value.sent_disco_ping.get(),
            sent_disco_pong: value.sent_disco_pong.get(),
            sent_disco_call_me_maybe: value.sent_disco_call_me_maybe.get(),
            recv_disco_bad_key: value.recv_disco_bad_key.get(),
            recv_disco_bad_parse: value.recv_disco_bad_parse.get(),
            recv_disco_udp: value.recv_disco_udp.get(),
            recv_disco_relay: value.recv_disco_relay.get(),
            recv_disco_ping: value.recv_disco_ping.get(),
            recv_disco_pong: value.recv_disco_pong.get(),
            recv_disco_call_me_maybe: value.recv_disco_call_me_maybe.get(),
            recv_disco_call_me_maybe_bad_disco: value.recv_disco_call_me_maybe_bad_disco.get(),
            relay_home_change: value.relay_home_change.get(),
            num_direct_conns_added: value.num_direct_conns_added.get(),
            num_direct_conns_removed: value.num_direct_conns_removed.get(),
            num_relay_conns_added: value.num_relay_conns_added.get(),
            num_relay_conns_removed: value.num_relay_conns_removed.get(),
            actor_tick_main: value.actor_tick_main.get(),
            actor_tick_msg: value.actor_tick_msg.get(),
            actor_tick_re_stun: value.actor_tick_re_stun.get(),
            actor_tick_portmap_changed: value.actor_tick_portmap_changed.get(),
            actor_tick_direct_addr_heartbeat: value.actor_tick_direct_addr_heartbeat.get(),
            actor_tick_direct_addr_update_receiver: value
                .actor_tick_direct_addr_update_receiver
                .get(),
            actor_link_change: value.actor_link_change.get(),
            actor_tick_other: value.actor_tick_other.get(),
            nodes_contacted: value.nodes_contacted.get(),
            nodes_contacted_directly: value.nodes_contacted_directly.get(),
            connection_handshake_success: value.connection_handshake_success.get(),
            connection_became_direct: value.connection_became_direct.get(),
        }
    }
}

#[derive(uniffi::Object)]
/// Metrics collected by net reports.
pub struct NetReportMetrics {
    /// Incoming STUN packets dropped due to a full receiving queue.
    pub stun_packets_dropped: u64,
    /// Number of IPv4 STUN packets sent.
    pub stun_packets_sent_ipv4: u64,
    /// Number of IPv6 STUN packets sent.
    pub stun_packets_sent_ipv6: u64,
    /// Number of IPv4 STUN packets received.
    pub stun_packets_recv_ipv4: u64,
    /// Number of IPv6 STUN packets received.
    pub stun_packets_recv_ipv6: u64,
    /// Number of reports executed by net_report, including full reports.
    pub reports: u64,
    /// Number of full reports executed by net_report
    pub reports_full: u64,
}

impl From<Arc<iroh::metrics::NetReportMetrics>> for NetReportMetrics {
    fn from(value: Arc<iroh::metrics::NetReportMetrics>) -> Self {
        Self {
            stun_packets_dropped: value.stun_packets_dropped.get(),
            stun_packets_sent_ipv4: value.stun_packets_sent_ipv4.get(),
            stun_packets_sent_ipv6: value.stun_packets_sent_ipv6.get(),
            stun_packets_recv_ipv4: value.stun_packets_recv_ipv4.get(),
            stun_packets_recv_ipv6: value.stun_packets_recv_ipv6.get(),
            reports: value.reports.get(),
            reports_full: value.reports_full.get(),
        }
    }
}

#[derive(uniffi::Object)]
/// Metrics collected by the portmapper service.
pub struct PortmapMetrics {
    /*
     * General port mapping metrics
     */
    /// Number of probing tasks started.
    pub probes_started: u64,
    /// Number of updates to the local port.
    pub local_port_updates: u64,
    /// Number of mapping tasks started.
    pub mapping_attempts: u64,
    /// Number of failed mapping tasks.
    pub mapping_failures: u64,
    /// Number of times the external address obtained via port mapping was updated.
    pub external_address_updated: u64,

    /*
     * UPnP metrics
     */
    /// Number of UPnP probes executed.
    pub upnp_probes: u64,
    /// Number of failed Upnp probes.
    pub upnp_probes_failed: u64,
    /// Number of UPnP probes that found it available.
    pub upnp_available: u64,
    /// Number of UPnP probes that resulted in a gateway different to the previous one,
    pub upnp_gateway_updated: u64,

    /*
     * PCP metrics
     */
    /// Number of PCP probes executed.
    pub pcp_probes: u64,
    /// Number of PCP probes that found it available.
    pub pcp_available: u64,
}

impl From<Arc<iroh::metrics::PortmapMetrics>> for PortmapMetrics {
    fn from(value: Arc<iroh::metrics::PortmapMetrics>) -> Self {
        Self {
            probes_started: value.probes_started.get(),
            local_port_updates: value.local_port_updates.get(),
            mapping_attempts: value.mapping_attempts.get(),
            mapping_failures: value.mapping_failures.get(),
            external_address_updated: value.external_address_updated.get(),
            upnp_probes: value.upnp_probes.get(),
            upnp_probes_failed: value.upnp_probes_failed.get(),
            upnp_available: value.upnp_available.get(),
            upnp_gateway_updated: value.upnp_gateway_updated.get(),
            pcp_probes: value.pcp_probes.get(),
            pcp_available: value.pcp_available.get(),
        }
    }
}
