# tests that correspond to the `src/doc.rs` rust api
import tempfile
import pytest
import asyncio
import iroh

from iroh import Iroh, ShareMode, LiveEventType, AddrInfoOptions, NodeOptions, NodeOptions, NodeAddr, PublicKey

@pytest.mark.asyncio
async def test_basic_sync():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    options = NodeOptions()
    options.enable_docs = True

    # Create node_0
    iroh_dir_0 = tempfile.TemporaryDirectory()
    node_0 = await Iroh.persistent_with_options(iroh_dir_0.name, options)

    # Create node_1
    iroh_dir_1 = tempfile.TemporaryDirectory()
    node_1 = await Iroh.persistent_with_options(iroh_dir_1.name, options)

    # Create doc on node_0
    doc_0 = await node_0.docs().create()
    ticket = await doc_0.share(ShareMode.WRITE, AddrInfoOptions.RELAY_AND_ADDRESSES)

    class SubscribeCallback:
        def __init__(self, found_s):
            self.found_s = found_s

        async def event(self, event):
            print("", event.type())
            await self.found_s.put(event)

    # Subscribe to sync events
    found_s_0 = asyncio.Queue(maxsize=1)
    cb0 = SubscribeCallback(found_s_0)
    await doc_0.subscribe(cb0)

    # Join the same doc from node_1
    found_s_1 = asyncio.Queue(maxsize=1)
    cb1 = SubscribeCallback(found_s_1)
    doc_1 = await node_1.docs().join_and_subscribe(ticket, cb1)

    # wait for initial sync
    while (True):
        event = await found_s_1.get()
        if (event.type() == LiveEventType.SYNC_FINISHED):
            break

    # Create author on node_1
    author = await node_1.authors().create()
    await doc_1.set_bytes(author, b"hello", b"world")

    # Wait for the content ready event
    while (True):
        event = await found_s_0.get()
        if (event.type() == LiveEventType.CONTENT_READY):
            hash = event.as_content_ready()

            # Get content from hash
            val = await node_1.blobs().read_to_bytes(hash)
            assert b"world" == val
            break

@pytest.mark.asyncio
async def test_custom_protocol():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    class MyProtocol:
        async def accept(self, connecting):
            conn = await connecting.connect()
            remote = conn.get_remote_node_id()
            print("accepting from ", remote)
            bi = await conn.accept_bi()

            bytes = await bi.recv().read_to_end(64)
            print("got", bytes)
            assert b"yo", bytes
            await bi.send().write_all(b"hello")
            await bi.send().finish()
            await bi.send().stopped()

        async def shutdown(self):
            print("shutting down")

    class ProtocolCreator:
        def create(self, endpoint, client):
            return MyProtocol()

    protocols = {}
    protocols[b"example/protocol/0"] = ProtocolCreator()

    options = NodeOptions()
    options.protocols = protocols

    # Create node_0
    node_1 = await Iroh.memory_with_options(options)

    # Create node_1
    node_2 = await Iroh.memory_with_options(options)

    alpn = b"example/protocol/0"
    node_id = await node_1.net().node_id()

    endpoint = node_2.node().endpoint()

    node_addr = NodeAddr(PublicKey.from_string(node_id), None, [])
    conn = await endpoint.connect(node_addr, alpn)
    remote = conn.get_remote_node_id()
    print("", remote)

    bi = await conn.open_bi()

    await bi.send().write_all(b"yo")
    await bi.send().finish()
    await bi.send().stopped()

    out = await bi.recv().read_exact(5)
    print("", out)
    assert b"hello", out

    await node_2.node().shutdown(False)
    await node_1.node().shutdown(False)
