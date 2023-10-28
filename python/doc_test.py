# tests that correspond to the `src/doc.rs` rust api
from iroh import PublicKey, SocketAddr, PeerAddr, Ipv4Addr, Ipv6Addr, iroh, AuthorId, NamespaceId, DocTicket, GetFilter
import pytest

def test_peer_addr():
    #
    # create a node_id
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
    expect_addrs = [ipv4, ipv6]
    peer_addr = PeerAddr(node_id, derp_region, expect_addrs)
    #
    # test we have returned the expected addresses
    got_addrs = peer_addr.direct_addresses()
    for (got, expect) in zip(got_addrs, expect_addrs):
        assert got.equal(expect)
        assert expect.equal(got)
    
    assert peer_addr.derp_region() == derp_region

def test_namespace_id():
    #
    # create id from string
    namespace_str = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
    namespace = NamespaceId.from_string(namespace_str)
    #
    # call to_string, ensure equal
    assert namespace.to_string() == namespace_str
    #
    # create another id, same string
    namespace_0 = NamespaceId.from_string(namespace_str)
    #
    # ensure equal
    assert namespace.equal(namespace_0)
    assert namespace_0.equal(namespace)

def test_author_id():
    #
    # create id from string
    author_str = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
    author = AuthorId.from_string(author_str)
    #
    # call to_string, ensure equal
    assert author.to_string() == author_str
    #
    # create another id, same string
    author_0 = AuthorId.from_string(author_str)
    #
    # ensure equal
    assert author.equal(author_0)
    assert author_0.equal(author)

def test_doc_ticket():
    #
    # create id from string
    doc_ticket_str = "ljapn77ljjzwrtxh4b35xg57gfvcrvey6ofrulgzuddnohwc2qnqcicshr4znowxoqsosz4gz55hebirkm32lncwltjfkbva6kl3denf5iaqcbiajjeteswek4ambkabzpcfoajganyabbz2zplaaaaaaaaaagrjyvlqcjqdoaaioowl2ygi2likyov62rofk4asma3qacdtvs6wrg7f7hkxlg3mlrkx"
    doc_ticket = DocTicket.from_string(doc_ticket_str)
    #
    # call to_string, ensure equal
    assert doc_ticket.to_string() == doc_ticket_str
    #
    # create another id, same string
    doc_ticket_0 = DocTicket.from_string(doc_ticket_str)
    #
    # ensure equal
    assert doc_ticket.equal(doc_ticket_0)
    assert doc_ticket_0.equal(doc_ticket)

def test_get_filter():
    #
    # all
    all = GetFilter.all()
    #
    # key
    key = GetFilter.key(b'key')
    key_0 = GetFilter.key(b'key')
    assert not all.equal(key)
    assert key_0.equal(key)
    #
    # prefix
    prefix = GetFilter.prefix(b'prefix')
    prefix_0 = GetFilter.prefix(b'prefix')
    assert not key.equal(prefix)
    assert prefix.equal(prefix_0)
    #
    # author
    author_str = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
    author = GetFilter.author(AuthorId.from_string(author_str))
    author_0 = GetFilter.author(AuthorId.from_string(author_str))
    assert not prefix.equal(author)
    assert author.equal(author_0)
    #
    # author&prefix
    author_prefix = GetFilter.author_prefix(AuthorId.from_string(author_str), b'prefix')
    author_prefix_0 = GetFilter.author_prefix(AuthorId.from_string(author_str), b'prefix')
    assert not author.equal(author_prefix)
    assert author_prefix.equal(author_prefix_0)
