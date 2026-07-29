"""Design A, same code: the same consumer program as `test_p3.py`, but loading
ONE combined `.so` instead of two.

The only difference between the two files is which package the modules come
from — and that is decided by `cdylib_name` at bindgen time, not by anything the
consumer writes. This is the evidence for "Design A does not close the door on
Design B".
"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out"))
from p3a.python import p3_iroh_core as core  # noqa: E402
from p3a.python import p3_iroh_ping_ffi as ping  # noqa: E402


async def main():
    server_ping = ping.Ping()
    client_ping = ping.Ping()

    server_ep = await core.Endpoint.bind(
        core.EndpointOptions(protocols={server_ping.alpn(): server_ping})
    )
    client_ep = await core.Endpoint.bind(core.EndpointOptions())

    print("server id       :", server_ep.id())
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
    print("OK: real iroh ping across one combined .so")


asyncio.run(main())
