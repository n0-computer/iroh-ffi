# tests that correspond to the `src/doc.rs` rust api
from iroh import Iroh, PublicKey, NodeAddr, AuthorId, Query, SortBy, SortDirection, QueryOptions, path_to_key, key_to_path
import pytest
import tempfile
import os
import random
import asyncio
import iroh

def test_node_addr():
    #
    # create a node_id
    key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
    node_id = PublicKey.from_string(key_str)
    #
    # create socketaddrs
    ipv4 = "127.0.0.1:3000"
    ipv6 = "::1:3000"
    #
    # relay url
    relay_url = "https://example.com"
    #
    # create a NodeAddr
    expect_addrs = [ipv4, ipv6]
    node_addr = NodeAddr(node_id, relay_url, expect_addrs)
    #
    # test we have returned the expected addresses
    got_addrs = node_addr.direct_addresses()
    for (got, expect) in zip(got_addrs, expect_addrs):
        assert got == expect

    assert relay_url == node_addr.relay_url()

def test_author_id():
    #
    # create id from string
    author_str = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
    author = AuthorId.from_string(author_str)
    #
    # call to_string, ensure equal
    assert str(author) == author_str
    #
    # create another id, same string
    author_0 = AuthorId.from_string(author_str)
    #
    # ensure equal
    assert author.equal(author_0)
    assert author_0.equal(author)

def test_query():
    opts = QueryOptions(sort_by=SortBy.KEY_AUTHOR, direction=SortDirection.ASC, offset=10, limit=10)
    # all
    all = Query.all(opts)
    assert 10 == all.offset()
    assert 10 == all.limit()

    # single_latest_per_key
    opts.direction = SortDirection.DESC
    opts.limit = 0
    opts.offset = 0
    single_latest_per_key = Query.single_latest_per_key(opts)
    assert 0 == single_latest_per_key.offset()
    assert single_latest_per_key.limit() is None

    # author
    opts.direction = SortDirection.ASC
    opts.offset = 100
    author = Query.author(AuthorId.from_string("mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"), opts)
    assert 100 == author.offset()
    assert author.limit() is None

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
    )
    assert 0 == key_prefix.offset()
    assert 100 == key_prefix.limit()

@pytest.mark.asyncio
async def test_doc_entry_basics():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    #
    # create node
    dir = tempfile.TemporaryDirectory()
    node = await Iroh.persistent(dir.name)
    #
    # create doc and author
    doc = await node.docs().create()
    author = await node.authors().create()
    #
    # create entry
    val = b'hello world!'
    key = b'foo'
    hash = await doc.set_bytes(author, key, val)
    #
    # get entry
    query = Query.author_key_exact(author, key)
    entry = await doc.get_one(query)
    assert hash.equal(entry.content_hash())
    assert len(val) == entry.content_len()
    got_val = await entry.content_bytes(doc)
    assert val == got_val

@pytest.mark.asyncio
async def test_doc_import_export():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

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
    node = await Iroh.persistent(iroh_dir.name)
    #
    # create doc and author
    doc = await node.docs().create()
    author = await node.authors().create()
    #
    # import entry
    key = path_to_key(path, None, in_root)
    await doc.import_file(author, key, path, True, None)
    #
    # get entry
    query = Query.author_key_exact(author, key)
    entry = await doc.get_one(query)
    #
    # export entry
    path = key_to_path(key, None, out_root)
    await doc.export_file(entry, path, None)
    #
    # read file
    file = open(path, "rb")
    got_bytes = file.read()
    file.close()
    #
    #
    assert bytes == got_bytes
