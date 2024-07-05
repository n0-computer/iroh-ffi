# tests that correspond to the `src/doc.rs` rust api
import tempfile
import pytest
import asyncio
import iroh

from iroh import IrohNode, ShareMode, LiveEventType, AddrInfoOptions

@pytest.mark.asyncio
async def test_basic_sync():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    # Create node_0
    iroh_dir_0 = tempfile.TemporaryDirectory()
    node_0 = await IrohNode.create(iroh_dir_0.name)

    # Create node_1
    iroh_dir_1 = tempfile.TemporaryDirectory()
    node_1 = await IrohNode.create(iroh_dir_1.name)

    # Create doc on node_0
    doc_0 = await node_0.doc_create()
    ticket = await doc_0.share(ShareMode.WRITE, AddrInfoOptions.RELAY_AND_ADDRESSES)

    class SubscribeCallback:
        def __init__(self, found_s):
            self.found_s = found_s

        async def event(self, event):
            print("", event.type())
            if (event.type() == LiveEventType.CONTENT_READY):
                print("got event type content ready")
                await self.found_s.put(event.as_content_ready())

    # Subscribe to sync events
    found_s = asyncio.Queue(maxsize=1)
    cb = SubscribeCallback(found_s)
    await doc_0.subscribe(cb)

    # Join the same doc from node_1
    doc_1 = await node_1.doc_join(ticket)

    # Create author on node_1
    author = await node_1.author_create()
    await doc_1.set_bytes(author, b"hello", b"world")

    # Wait for the content ready event
    hash = await found_s.get()
    found_s.task_done()

    # Get content from hash
    val = await node_1.blobs_read_to_bytes(hash)
    assert b"world" == val
