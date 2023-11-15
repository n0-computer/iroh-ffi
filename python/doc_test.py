# tests that correspond to the `src/doc.rs` rust api
from iroh import IrohNode, PublicKey, SocketAddr, NodeAddr, Ipv4Addr, Ipv6Addr, iroh, AuthorId, NamespaceId, DocTicket, Query, SortBy, SortDirection, QueryOptions, path_to_key, key_to_path
import pytest
import tempfile
import os
import random

def test_node_addr():
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
    # create a NodeAddr
    expect_addrs = [ipv4, ipv6]
    node_addr = NodeAddr(node_id, derp_region, expect_addrs)
    #
    # test we have returned the expected addresses
    got_addrs = node_addr.direct_addresses()
    for (got, expect) in zip(got_addrs, expect_addrs):
        assert got.equal(expect)
        assert expect.equal(got)
    
    assert derp_region == node_addr.derp_region()

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
    doc_ticket_str = "docaaqjjfgbzx2ry4zpaoujdppvqktgvfvpxgqubkghiialqovv7z4wosqbebpvjjp2tywajvg6unjza6dnugkalg4srmwkcucmhka7mgy4r3aa4aibayaeusjsjlcfoagavaa4xrcxaetag4aaq45mxvqaaaaaaaaadiu4kvybeybxaaehhlf5mdenfufmhk7nixcvoajganyabbz2zplgbno2vsnuvtkpyvlqcjqdoaaioowl22k3fc26qjx4ot6fk4"
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

def test_query():
    opts = QueryOptions(SortBy.KEY_AUTHOR, SortDirection.ASC, 10, 10)
    # all
    all = Query.all(opts)
    assert 10 == all.offset()
    assert 10 == all.limit()

    # single_latest_per_key
    opts.direction = SortDirection.DESC
    opts.limit = 0
    opts.offset = 0
    single_latest_per_key = Query.single_latest_per_key(opts);
    assert 0 == single_latest_per_key.offset()
    assert None == single_latest_per_key.limit()

    # author
    opts.direction = SortDirection.ASC
    opts.offset = 100 
    author = Query.author(AuthorId.from_string("mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"), opts)
    assert 100 == author.offset()
    assert None == author.limit()

    # key_exact
    opts.sort_by = SortBy.KEY_AUTHOR
    opts.direction = SortDirection.DESC
    opts.offset = 0
    opts.limit = 100
    key_exact = Query.key_exact(
        b'key',
        opts
    )
    assert 0 == key_exact.offset()
    assert 100 == key_exact.limit()

    # key_prefix
    key_prefix = Query.key_prefix(
        b'prefix',
        opts
    );
    assert 0 == key_prefix.offset()
    assert 100 == key_prefix.limit()

def test_doc_entry_basics():
    #
    # create node
    dir = tempfile.TemporaryDirectory()
    node = IrohNode(dir.name)
    #
    # create doc and author
    doc = node.doc_create()
    author = node.author_create()
    #
    # create entry
    val = b'hello world!'
    key = b'foo'
    hash = doc.set_bytes(author, key, val)
    #
    # get entry
    query = Query.author_key_exact(author, key)
    entry = doc.get_one(query)
    assert hash.equal(entry.content_hash())
    got_val = doc.read_to_bytes(entry)
    assert val == got_val
    assert len(val) == entry.content_len()

def test_doc_import_export():
    #
    # create file temp der
    dir = tempfile.TemporaryDirectory()
    in_root = os.path.join(dir.name, "in")
    out_root = os.path.join(dir.name, "out")
    os.makedirs(in_root, exist_ok=True)
    os.makedirs(out_root, exist_ok=True)
    #
    # create file
    path = os.path.join(in_root, "test")
    size = 100
    bytes = bytearray(map(random.getrandbits,(8,)*size))
    file = open(path, "wb")
    file.write(bytes)
    file.close()
    #
    # create node
    iroh_dir = tempfile.TemporaryDirectory()
    node = IrohNode(iroh_dir.name)
    #
    # create doc and author
    doc = node.doc_create()
    author = node.author_create()
    #
    # import entry
    key = path_to_key(path, None, in_root)
    doc.import_file(author, key, path, True, None)
    #
    # get entry
    query = Query.author_key_exact(author, key)
    entry = doc.get_one(query)
    #
    # export entry
    path = key_to_path(key, None, out_root)
    doc.export_file(entry, path, None)
    #
    # read file
    file = open(path, "rb")
    got_bytes = file.read()
    file.close()
    #
    #
    assert bytes == got_bytes

