"""Design B, for real: iroh-ping over a real QUIC connection, where `Endpoint`
lives in one shipped .so and the ping protocol lives in another.

Note that the *consumer* assembles the two: the ping library exports a `Ping`
that is a `ProtocolCreator`, and Python hands it to `Endpoint.bind` under its
ALPN. Nothing in the ping library spawns a router or owns an endpoint.
"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out"))
from p3.python import p3_iroh_core as core  # noqa: E402
from p3.python import p3_iroh_ping_ffi as ping  # noqa: E402


async def main():
    server_ping = ping.Ping()
    client_ping = ping.Ping()

    # This is the composition point: a protocol from one library, registered on
    # an endpoint from another, entirely from the consumer's language.
    server_ep = await core.Endpoint.bind(
        core.EndpointOptions(protocols={server_ping.alpn(): server_ping})
    )
    client_ep = await core.Endpoint.bind(core.EndpointOptions())

    print("server id       :", server_ep.id())
    print("core static addr:", hex(server_ep.core_static_addr()))
    assert server_ping.alpn() == b"iroh/ping/0"

    addrs = server_ep.direct_addrs()
    print("server addrs    :", addrs)

    rtt = await client_ping.ping(client_ep, server_ep.id(), addrs)
    print("ping rtt        :", rtt, "us")
    print("pings sent      :", client_ping.pings_sent())
    print("pings received  :", server_ping.pings_received())
    assert client_ping.pings_sent() == 1
    assert server_ping.pings_received() == 1

    await client_ep.close()
    await server_ep.close()
    print("OK: real iroh ping across two separately-shipped .so files")


asyncio.run(main())
