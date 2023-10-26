# tests that correspond to the `src/doc.rs` rust api
from iroh import PublicKey, SocketAddr, PeerAddr, Ipv4Addr, Ipv6Addr

def test_peer_addr():
    # test the node_id
    # test the socketaddr

    # create a publickey
    key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
    node_id = PublicKey.from_string(key_str)
    #
    # create socketaddrs
    ipv4_ip = Ipv4Addr.from_string("127.0.0.1")
    ipv6_ip = Ipv6Addr.from_string("::1")
    port = 3000
    #
    # create socket addrs
    ipv4 = SocketAddr.from_ipv4(ipv4_ip, port)
    ipv6 = SocketAddr.from_ipv6(ipv6_ip, port)
    #
    # derp region
    derp_region = 1
    #
    # create a PeerAddr
    peer_addr = PeerAddr(node_id, derp_region, [ipv4, ipv6])
    #
    # test we have proper addrs
    # assert peer_addr.direct_addresses() == [ipv4, ipv6]
    # assert peer_addr.derp_region() == derp_region


