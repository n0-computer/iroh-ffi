# tests that correspond to the `src/doc.rs` rust api
import tempfile
import pytest
import asyncio
import iroh

from iroh import Iroh, ShareMode, LiveEventType, AddrInfoOptions

@pytest.mark.asyncio
async def test_basic_sync():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    # Create node_0
    iroh_dir_0 = tempfile.TemporaryDirectory()
    node_0 = await Iroh.persistent(iroh_dir_0.name)

    # Create node_1
    iroh_dir_1 = tempfile.TemporaryDirectory()
    node_1 = await Iroh.persistent(iroh_dir_1.name)

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
