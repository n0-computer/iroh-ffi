# Tests that correspond to the `src/gossip.rs` and `src/router.rs` rust api.
#
# Run with the RTT numbers visible:
#
#     python -m pytest gossip_test.py --log-cli-level=INFO
#
import asyncio
import logging
import statistics
import time

from iroh import (
    AppRouterBuilder,
    Endpoint,
    EndpointOptions,
    GossipNode,
    RelayMode,
    preset_n0,
)

LOG = logging.getLogger(__name__)

# Deterministic 32-byte topic; gossip requires exactly 32 bytes.
TOPIC = bytes(range(32))
DIRECT_ALPN = b"iroh-ffi/test/rtt/0"

# Joining a topic has to traverse discovery + a QUIC handshake, so give it room.
JOIN_TIMEOUT = 30
MSG_TIMEOUT = 30

RTT_SAMPLES = 10


async def _bind():
    """A relay-free endpoint; the tests dial directly over loopback."""
    return await Endpoint.bind(
        EndpointOptions(preset=preset_n0(), relay_mode=RelayMode.disabled())
    )


async def _spawn_gossip(ep, custom_alpn=None):
    """Spawn gossip behind an AppRouter.

    `GossipNode` alone does not accept inbound connections -- the router is what
    binds the gossip ALPN to the endpoint. Custom ALPNs must be registered
    before `spawn()`, which consumes the builder.
    """
    gossip = await GossipNode.spawn(ep)
    builder = AppRouterBuilder(ep)
    await builder.accept_gossip(gossip)
    receiver = None
    if custom_alpn is not None:
        receiver = await builder.accept_custom_alpn(custom_alpn)
    router = await builder.spawn()
    return gossip, router, receiver


async def _recv_matching(topic, predicate, timeout=MSG_TIMEOUT):
    """Pull messages until one satisfies `predicate`, or time out.

    Gossip delivers control traffic and unrelated payloads on the same stream,
    so a bare `next_message()` is not enough to correlate a reply.
    """

    async def _pump():
        while True:
            msg = await topic.next_message()
            if msg is None:
                raise AssertionError("gossip stream ended before a match")
            if predicate(msg):
                return msg

    return await asyncio.wait_for(_pump(), timeout=timeout)


async def test_gossip_broadcast_is_received():
    """A broadcasts, B receives -- exercises the router, gossip, and sender id."""
    ep_a = await _bind()
    ep_b = await _bind()
    a_id = ep_a.id()

    gossip_a, router_a, _ = await _spawn_gossip(ep_a)
    gossip_b, router_b, _ = await _spawn_gossip(ep_b)

    # B bootstraps off A's address; A waits for B to find it.
    topic_a = await gossip_a.subscribe(TOPIC, [])
    topic_b = await gossip_b.subscribe(TOPIC, [ep_a.addr()])

    await asyncio.wait_for(topic_b.wait_to_join(), timeout=JOIN_TIMEOUT)
    await asyncio.wait_for(topic_a.wait_to_join(), timeout=JOIN_TIMEOUT)

    await topic_a.broadcast(b"hello gossip")

    msg = await _recv_matching(topic_b, lambda m: m.content == b"hello gossip")
    assert msg.content == b"hello gossip"
    # The sender is a real EndpointId object, not a string.
    assert msg.sender == a_id
    assert msg.sender.to_bytes() == a_id.to_bytes()
    LOG.info("received %r from %s", msg.content, msg.sender.fmt_short())

    await router_a.shutdown()
    await router_b.shutdown()
    await ep_a.close()
    await ep_b.close()


async def test_gossip_rtt_tracking():
    """Record gossip application-level round-trip against transport RTT.

    Two different numbers, deliberately measured side by side:

      * app-level: broadcast -> peer echoes -> we see the echo. Includes gossip
        forwarding, scheduling and the Python/FFI hop.
      * transport: `Connection.rtt()` on a direct QUIC connection between the
        same two endpoints -- what QUIC's own estimator reports.
    """
    ep_a = await _bind()
    ep_b = await _bind()

    gossip_a, router_a, _ = await _spawn_gossip(ep_a)
    gossip_b, router_b, receiver_b = await _spawn_gossip(ep_b, custom_alpn=DIRECT_ALPN)

    topic_a = await gossip_a.subscribe(TOPIC, [])
    topic_b = await gossip_b.subscribe(TOPIC, [ep_a.addr()])

    await asyncio.wait_for(topic_b.wait_to_join(), timeout=JOIN_TIMEOUT)
    await asyncio.wait_for(topic_a.wait_to_join(), timeout=JOIN_TIMEOUT)

    stop = asyncio.Event()

    async def echo():
        while not stop.is_set():
            try:
                msg = await topic_b.next_message()
            except Exception:
                return
            if msg is None:
                return
            if msg.content.startswith(b"ping:"):
                await topic_b.broadcast(b"pong:" + msg.content[5:])

    echo_task = asyncio.create_task(echo())

    samples_ms = []
    for i in range(RTT_SAMPLES):
        token = str(i).encode()
        expected = b"pong:" + token
        t0 = time.perf_counter()
        await topic_a.broadcast(b"ping:" + token)
        msg = await _recv_matching(topic_a, lambda m, e=expected: m.content == e)
        elapsed_ms = (time.perf_counter() - t0) * 1000.0
        samples_ms.append(elapsed_ms)
        LOG.info(
            "gossip app-level rtt sample %2d: %8.3f ms  (echo from %s)",
            i,
            elapsed_ms,
            msg.sender.fmt_short(),
        )

    stop.set()
    echo_task.cancel()

    LOG.info(
        "gossip app-level rtt over %d samples: min=%.3f ms median=%.3f ms max=%.3f ms",
        len(samples_ms),
        min(samples_ms),
        statistics.median(samples_ms),
        max(samples_ms),
    )

    # Transport-level RTT on a direct connection between the same two endpoints.
    async def accept_direct():
        return await receiver_b.next_connection()

    accept_task = asyncio.create_task(accept_direct())
    conn = await asyncio.wait_for(ep_a.connect(ep_b.addr(), DIRECT_ALPN), timeout=30)
    server_conn = await asyncio.wait_for(accept_task, timeout=30)
    assert server_conn is not None

    # `Connection.rtt()` only reports the *selected* path, and a freshly
    # established connection has none yet -- it returns None until path
    # validation settles. Any RTT tracking built on this must poll, not
    # sample once. Drive some traffic and wait for selection.
    transport_rtt_us = None
    for attempt in range(50):
        conn.send_datagram(b"rtt-probe")
        transport_rtt_us = conn.rtt_us()
        if transport_rtt_us is not None:
            LOG.info("transport rtt available after %d poll(s)", attempt + 1)
            break
        await asyncio.sleep(0.1)

    LOG.info(
        "transport Connection.rtt(): %s ms / rtt_us(): %s us",
        conn.rtt(),
        transport_rtt_us,
    )
    for path in conn.paths():
        LOG.info(
            "  path %s selected=%s relay=%s rtt_ms=%d rtt_us=%d",
            path.remote_addr,
            path.is_selected,
            path.is_relay,
            path.rtt_ms,
            path.rtt_us,
        )

    assert transport_rtt_us is not None, "no path selected after polling"
    LOG.info(
        "SUMMARY: gossip app-level median %.3f ms (%.0f us) "
        "vs transport %d us -- gossip overhead ~%.0f us",
        statistics.median(samples_ms),
        statistics.median(samples_ms) * 1000.0,
        transport_rtt_us,
        statistics.median(samples_ms) * 1000.0 - transport_rtt_us,
    )

    # Sanity only -- the point of this test is the recorded numbers, not a bound.
    assert len(samples_ms) == RTT_SAMPLES
    assert all(ms >= 0 for ms in samples_ms)

    conn.close(0, b"bye")
    await router_a.shutdown()
    await router_b.shutdown()
    await ep_a.close()
    await ep_b.close()
