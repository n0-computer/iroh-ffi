"""
Minimal iroh-ffi Python demo (peer-to-peer; no client/server framing).

Run the accept side in one terminal:

    python main.py accept

Copy the printed ticket and run the dial side in another:

    python main.py connect <ticket>

The dial side opens a bidirectional QUIC stream, sends `hello`, and prints
what the accept side echoes back.
"""

import argparse
import asyncio

import iroh

ALPN = b"iroh-ffi/example/0"


async def accept(ep):
    print("listening on:", ep.id())
    ticket = iroh.EndpointTicket.from_addr(ep.addr())
    print("ticket:", str(ticket))

    incoming = await ep.accept_next()
    assert incoming is not None, "endpoint closed before any connection arrived"
    accepting = await incoming.accept()
    conn = await accepting.connect()
    print("accepted connection from", str(conn.remote_id()))

    bi = await conn.accept_bi()
    recv = bi.recv()
    send = bi.send()

    data = await recv.read_to_end(1024)
    print("echo side received:", data.decode("utf8"))

    await send.write_all(data)
    await send.finish()
    await asyncio.sleep(0.5)
    await ep.close()


async def connect(ep, ticket_str):
    ticket = iroh.EndpointTicket.from_string(ticket_str)
    addr = ticket.endpoint_addr()
    print("dialing", str(addr.id()))
    conn = await ep.connect(addr, list(ALPN))

    bi = await conn.open_bi()
    send = bi.send()
    recv = bi.recv()
    await send.write_all(list(b"hello"))
    await send.finish()

    echoed = await recv.read_to_end(1024)
    print("dial side received:", echoed.decode("utf8"))
    await ep.close()


async def main():
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    parser = argparse.ArgumentParser(description="iroh-ffi Python demo")
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("accept", help="bind an endpoint and accept one connection")
    p_connect = sub.add_parser("connect", help="dial the ticket")
    p_connect.add_argument("ticket", help="EndpointTicket string")
    args = parser.parse_args()

    opts = iroh.EndpointOptions(
        alpns=[list(ALPN)] if args.cmd == "accept" else None,
    )
    ep = await iroh.Endpoint.bind(opts)

    if args.cmd == "accept":
        await accept(ep)
    else:
        await connect(ep, args.ticket)


if __name__ == "__main__":
    asyncio.run(main())
