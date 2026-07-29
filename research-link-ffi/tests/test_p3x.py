"""Design B, packaged the way it would actually ship: two SEPARATE packages in
separate directories, wired only by a relative `$ORIGIN/../iroh_core` rpath.

No `LD_LIBRARY_PATH`, no environment setup, no Rust toolchain. This is the test
that backs the hard requirement in the RFD: a consumer runs `pip install` and it
works. Shipping `libstd-*.so` is our packaging job, never theirs.
"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out"))
from p3x.iroh_core import p3_iroh_core as core  # noqa: E402
from p3x.iroh_ping import p3_iroh_ping_ffi as ping  # noqa: E402


async def main():
    assert "LD_LIBRARY_PATH" not in os.environ, "run this WITHOUT LD_LIBRARY_PATH"

    server_ping = ping.Ping()
    client_ping = ping.Ping()

    server_ep = await core.Endpoint.bind(
        core.EndpointOptions(protocols={server_ping.alpn(): server_ping})
    )
    client_ep = await core.Endpoint.bind(core.EndpointOptions())

    rtt = await client_ping.ping(client_ep, server_ep.id(), server_ep.direct_addrs())
    print("ping rtt        :", rtt, "us")
    print("pings received  :", server_ping.pings_received())
    assert server_ping.pings_received() == 1

    await client_ep.close()
    await server_ep.close()
    print("OK: two separate packages, separate dirs, no env vars")


asyncio.run(main())
