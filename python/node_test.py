# tests that correspond to the `src/doc.rs` rust api
import tempfile
import queue
import pytest

from iroh import IrohNode, ShareMode, LiveEventType, AddrInfoOptions

@pytest.mark.asyncio
async def test_basic_sync():
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

        def event(self, event):
            print("", event.type())
            if (event.type() == LiveEventType.CONTENT_READY):
                print("got event type content ready")
                self.found_s.put(event.as_content_ready())

    # Subscribe to sync events
    found_s = queue.Queue()
    cb = SubscribeCallback(found_s)
    await doc_0.subscribe(cb)

    # Join the same doc from node_1
    doc_1 = await node_1.doc_join(ticket)

    # Create author on node_1
    author = await node_1.author_create()
    await doc_1.set_bytes(author, b"hello", b"world")

    # Wait for the content ready event
    hash = found_s.get()

    # Get content from hash
    val = await node_1.blobs_read_to_bytes(hash)
    assert b"world" == val
