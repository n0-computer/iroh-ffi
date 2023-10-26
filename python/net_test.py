# tests that correspond to the `src/net.rs` rust api
from iroh import Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr, SocketAddrType

def test_ipv4_addr():
    #
    # create ipv4 addr from the constructor
    from_cons = Ipv4Addr(10,10,10,10)
    #
    # create ipv4 addr from a string
    ip_str = "10.10.10.10"
    from_str = Ipv4Addr.from_string(ip_str)
    #
    # ensure the strings are what we expect, 
    assert from_cons.to_string() == ip_str
    assert from_str.to_string() == ip_str
    #
    # ensure octets are what we expect
    octets = [10,10,10,10]
    assert from_cons.octets() == [10,10,10,10]
    assert from_str.octets() == [10,10,10,10]
    #
    # ensure equal method works
    assert from_cons.equal(from_str)
    assert from_str.equal(from_cons)

def test_ipv6_addr():
    #
    # create ipv6 addr from the constructor
    from_cons = Ipv6Addr(10000,10000,10000,10000,10000,10000,10000,10000)
    #
    # create ipv6 addr from a string
    ip_str = "2710:2710:2710:2710:2710:2710:2710:2710"
    from_str = Ipv6Addr.from_string(ip_str)
    #
    # ensure strings are what we expect, 
    assert from_cons.to_string() == ip_str
    assert from_str.to_string() == ip_str
    #
    # ensure segments are what we expect
    segments = [10000,10000,10000,10000,10000,10000,10000,10000]
    assert from_cons.segments() == segments
    assert from_str.segments() ==  segments
    #
    # ensure equal method works
    assert from_cons.equal(from_str)
    assert from_str.equal(from_cons)

def test_socket_addr_v4():
    #
    # create an addr and a port
    ipv4 = Ipv4Addr.from_string("127.0.0.1")
    port = 3000
    socket_addr_str = "127.0.0.1:3000"
    ip_str = "127.0.0.1"
    #
    # create a socket addrs
    from_cons = SocketAddrV4(ipv4, port)
    from_str = SocketAddrV4.from_string(socket_addr_str)
    #
    # test the ip addr and port are as expected
    assert from_cons.ip().to_string() == ip_str
    assert from_cons.port() == port
    #
    assert from_str.ip().to_string() == ip_str
    assert from_str.port() == port
    #
    # test that the to_string works as expected
    assert from_cons.to_string() == socket_addr_str
    assert from_str.to_string() == socket_addr_str
    #
    # ensure equal method works
    assert from_cons.equal(from_str)
    assert from_str.equal(from_cons)

def test_socket_addr_v6():
    #
    # create an addr and a port
    ipv6 = Ipv6Addr.from_string("::1")
    port = 3000
    socket_addr_str = "[::1]:3000"
    ip_str = "::1"
    #
    # create a socket addrs
    from_cons = SocketAddrV6(ipv6, port)
    from_str = SocketAddrV6.from_string(socket_addr_str)
    #
    # test the ip addr and port are as expected
    assert from_cons.ip().to_string() == ip_str
    assert from_cons.port() == port
    #
    assert from_str.ip().to_string() == ip_str
    assert from_str.port() == port
    #
    # test that the to_string works as expected
    assert from_cons.to_string() == socket_addr_str
    assert from_str.to_string() == socket_addr_str
    #
    # ensure equal method works
    assert from_cons.equal(from_str)
    assert from_str.equal(from_cons)

def test_socket_addr():
    #
    # create a ip addrs & port
    ipv4_ip = Ipv4Addr.from_string("127.0.0.1")
    ipv6_ip = Ipv6Addr.from_string("::1")
    port = 3000
    #
    # create socket addrs
    ipv4 = SocketAddr.from_ipv4(ipv4_ip, port)
    ipv6 = SocketAddr.from_ipv6(ipv6_ip, port)
    #
    # ensure the types are as expected
    assert ipv4.type() == SocketAddrType.V4
    assert ipv6.type() == SocketAddrType.V6
    #
    # ensure we can get the addrs out properly
    ipv4_addr = ipv4.as_ipv4()
    ipv6_addr = ipv6.as_ipv6()
    #
    # ensure they are as expected
    assert ipv4_addr.ip().to_string() == ipv4_ip.to_string()
    assert ipv6_addr.ip().to_string() == ipv6_ip.to_string()
    assert ipv4_addr.port() == port 
    assert ipv6_addr.port() == port
    #
    # ensure equal method works
    ipv4_other = SocketAddr.from_ipv4(ipv4_ip, port)
    ipv6_other = SocketAddr.from_ipv6(ipv6_ip, port)
    assert ipv4.equal(ipv4_other)
    assert ipv4_other.equal(ipv4)
    assert ipv6.equal(ipv6_other)
    assert ipv6_other.equal(ipv6)
