# tests that correspond to the `src/doc.rs` rust api
from iroh import Hash
import pytest

def test_hash():
    hash_str = "bafkr4ih6qxpyfyrgxbcrvmiqbm7hb5fdpn4yezj7ayh6gwto4hm2573glu"
    hex_str = "fe85df82e226b8451ab1100b3e70f4a37b7982653f060fe35a6ee1d9aeff665d"
    bytes = b'\xfe\x85\xdf\x82\xe2\x26\xb8\x45\x1a\xb1\x10\x0b\x3e\x70\xf4\xa3\x7b\x79\x82\x65\x3f\x06\x0f\xe3\x5a\x6e\xe1\xd9\xae\xff\x66\x5d'
    cid_prefix = b'\x01\x55\x1e\x20'
    #
    # create hash from string
    hash = Hash.from_string(hash_str)
    #
    # test methods are as expected
    assert hash.to_string() == hash_str
    assert hash.to_bytes() == bytes
    assert hash.to_hex() == hex_str 
    assert hash.as_cid_bytes() == cid_prefix + bytes
    #
    # create hash from bytes
    hash_0 = Hash.from_bytes(bytes)
    #
    # test methods are as expected
    assert hash_0.to_string() == hash_str
    assert hash_0.to_bytes() == bytes
    assert hash_0.to_hex() == hex_str 
    assert hash_0.as_cid_bytes() == cid_prefix + bytes
    #
    # create hash from cid bytes
    hash_1 = Hash.from_cid_bytes(cid_prefix + bytes)
    #
    # test methods are as expected
    assert hash_1.to_string() == hash_str
    assert hash_1.to_bytes() == bytes
    assert hash_1.to_hex() == hex_str 
    assert hash_1.as_cid_bytes() == cid_prefix + bytes
    #
    # test that the eq function works
    assert hash.equal(hash_0)
    assert hash.equal(hash_1)
    assert hash_0.equal(hash)
    assert hash_0.equal(hash_1)
    assert hash_1.equal(hash)
    assert hash_1.equal(hash_0)
 
#def test_peer_addr():
#    #
#    # create a node_id
#    key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
#    node_id = PublicKey.from_string(key_str)
#    #
#    # create socketaddrs
#    ipv4_ip = Ipv4Addr.from_string("127.0.0.1")
#    ipv6_ip = Ipv6Addr.from_string("::1")
#    port = 3000
#    #
#    # create socket addrs
#    ipv4 = SocketAddr.from_ipv4(ipv4_ip, port)
#    ipv6 = SocketAddr.from_ipv6(ipv6_ip, port)
#    #
#    # derp region
#    derp_region = 1
#    #
#    # create a PeerAddr
#    expect_addrs = [ipv4, ipv6]
#    peer_addr = PeerAddr(node_id, derp_region, expect_addrs)
#    #
#    # test we have returned the expected addresses
#    got_addrs = peer_addr.direct_addresses()
#    for (got, expect) in zip(got_addrs, expect_addrs):
#        assert got.equal(expect)
#        assert expect.equal(got)
    
#    assert peer_addr.derp_region() == derp_region

