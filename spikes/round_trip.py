"""SPIKE: prove an Endpoint minted by the `iroh` package's native library can be used by
the separately-built `iroh_ping` native library, with only one copy of iroh in the process.

Run from the staging dir:  python3 round_trip.py
"""

import asyncio

import iroh
import iroh_ping


async def main() -> None:
    # uniffi needs the running loop registered before any async FFI call (see python/conftest.py)
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())
    iroh_ping.iroh_ffi_ping.uniffi_set_event_loop(asyncio.get_running_loop())

    ping = iroh_ping.Ping()

    # Server: an endpoint from the CORE library, with a protocol handler from the PLUGIN
    # library registered on it. `handler()` returns an `iroh.iroh_ffi.ProtocolHandler` —
    # a callback-interface handle crossing between the two libraries.
    class PingCreator(iroh.ProtocolCreator):
        def create(self, endpoint):
            return ping.handler()

    server = await iroh.Endpoint.bind(
        iroh.EndpointOptions(
            preset=iroh.preset_minimal(),
            protocols={iroh_ping.alpn(): PingCreator()},
        )
    )
    server_addr = server.addr()
    print(f"  server bound: {server_addr}")

    # Client: a second endpoint from the CORE library, handed to the PLUGIN library.
    client = await iroh.Endpoint.bind(iroh.EndpointOptions(preset=iroh.preset_minimal()))
    print(f"  client bound: {client.addr()}")

    rtt_ms = await ping.ping(client, server_addr)
    print(f"  PING -> PONG round trip: {rtt_ms} ms")

    await client.close()
    await server.close()
    print("  OK: Endpoint crossed the library boundary and the protocol ran")


if __name__ == "__main__":
    asyncio.run(main())
