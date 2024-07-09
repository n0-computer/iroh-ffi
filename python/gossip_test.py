# tests that correspond to the `src/gossp.rs` rust api
import tempfile
import pytest
import asyncio
import iroh

from iroh import IrohNode, ShareMode, LiveEventType, MessageType, GossipMessageCallback, set_log_level, LogLevel

class Callback(GossipMessageCallback):
    def __init__(self, name):
        print("init", name)
        self.name = name
        self.chan = asyncio.Queue()

    async def on_message(self, msg):
        print("onmessage")
        print(self.name, msg.type())
        await self.chan.put(msg)

@pytest.mark.asyncio
async def test_gossip_basic():
    set_log_level(LogLevel.WARN)

    n0 = await IrohNode.memory()
    n1 = await IrohNode.memory()

    # Create a topic
    topic = bytearray([1] * 32)

    # Setup gossip on node 0
    cb0 = Callback("n0")
    n1_id = await n1.node_id()
    n1_addr = await n1.node_addr()
    await n0.add_node_addr(n1_addr)

    print("subscribe n0")
    sink0 = await n0.gossip_subscribe(topic, [n1_id], cb0)

    # Setup gossip on node 1
    cb1 = Callback("n1")
    n0_id = await n0.node_id()
    n0_addr = await n0.node_addr()
    await n1.add_node_addr(n0_addr)

    print("subscribe n1")
    sink1 = await n1.gossip_subscribe(topic, [n0_id], cb1)

    # Wait for n1 to show up for n0
    while (True):
        event = await cb0.chan.get()
        print("<<", event.type())
        if (event.type() == MessageType.NEIGHBOR_UP):
            assert event.as_neighbor_up() == n1_id
            break


    # Broadcact message from node 0
    print("broadcasting message")
    msg_content = bytearray("hello".encode("utf-8"))

    await sink0.broadcast(msg_content)

    # Wait for message on n1
    found = False

    # Wait for the message on node 1
    while (True):
        event = await cb1.chan.get()
        if (event.type() == MessageType.RECEIVED):
            msg = event.as_received()
            assert msg.content == msg_content
            assert msg.delivered_from == n0_id
            found = True
            break

    assert found
