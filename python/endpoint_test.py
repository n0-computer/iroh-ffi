# Tests that correspond to the `src/endpoint.rs` rust api.
import asyncio

import pytest

import iroh
from iroh import Endpoint, EndpointBuilder, EndpointOptions, EndpointTicket, RelayMode, preset_minimal, preset_n0

ALPN = b"iroh-ffi/test/0"


class CustomPreset(iroh.Preset):
    """A user-implemented Preset: minimal baseline + a custom ALPN."""

    def apply(self, builder):
        builder.apply_minimal()
        builder.alpns([b"custom/preset/1"])


async def test_custom_preset():
    ep = await Endpoint.bind(EndpointOptions(preset=CustomPreset()))
    assert len(ep.bound_sockets()) > 0
    await ep.close()


async def test_builder_bind():
    builder = EndpointBuilder()
    builder.apply_minimal()
    ep = await builder.bind()
    assert len(ep.bound_sockets()) > 0
    await ep.close()


async def test_builder_bind_consumes():
    builder = EndpointBuilder()
    builder.apply_minimal()
    ep = await builder.bind()
    await ep.close()
    with pytest.raises(Exception) as exc_info:
        await builder.bind()
    assert "already consumed" in repr(exc_info.value)


async def test_bind_lifecycle():
    ep = await Endpoint.bind(EndpointOptions(preset=preset_minimal()))
    id_ = ep.id()
    assert len(str(id_)) > 0
    assert ep.addr().id() == id_
    assert len(ep.bound_sockets()) > 0
    assert ep.secret_key().public().to_bytes() == id_.to_bytes()
    await ep.close()
    assert ep.is_closed()


async def test_endpoint_ticket_roundtrip():
    ep = await Endpoint.bind(EndpointOptions(preset=preset_minimal()))
    addr = ep.addr()
    ticket = EndpointTicket.from_addr(addr)
    s = str(ticket)
    assert s.startswith("endpoint")
    parsed = EndpointTicket.from_string(s)
    assert parsed.endpoint_addr().id() == addr.id()
    await ep.close()


def test_endpoint_ticket_rejects_garbage():
    with pytest.raises(Exception):
        EndpointTicket.from_string("not-a-ticket")


async def test_connect_echo_roundtrip():
    server = await Endpoint.bind(
        EndpointOptions(
            preset=preset_n0(),
            alpns=[ALPN],
            relay_mode=RelayMode.disabled(),
        )
    )
    server_addr = server.addr()
    server_id = server.id()

    async def serve():
        incoming = await server.accept_next()
        assert incoming is not None
        accepting = await incoming.accept()
        conn = await accepting.connect()
        assert conn.alpn() == ALPN
        bi = await conn.accept_bi()
        recv = bi.recv()
        send = bi.send()
        msg = await recv.read_to_end(64)
        await send.write_all(msg)
        await send.finish()
        dg = await conn.read_datagram()
        conn.send_datagram(dg)
        await conn.closed()

    server_task = asyncio.create_task(serve())

    client = await Endpoint.bind(
        EndpointOptions(preset=preset_n0(), relay_mode=RelayMode.disabled())
    )
    conn = await client.connect(server_addr, ALPN)
    assert conn.remote_id() == server_id
    assert len(conn.paths()) > 0

    bi = await conn.open_bi()
    send = bi.send()
    recv = bi.recv()
    await send.write_all(b"hello iroh")
    await send.finish()
    echoed = await recv.read_to_end(64)
    assert echoed == b"hello iroh"

    conn.send_datagram(b"ping")
    pong = await conn.read_datagram()
    assert pong == b"ping"

    stats = conn.stats()
    assert stats.udp_tx_datagrams > 0

    conn.close(0, b"bye")
    await server_task
    await client.close()
    await server.close()


async def test_uni_stream():
    server = await Endpoint.bind(
        EndpointOptions(
            preset=preset_n0(),
            alpns=[ALPN],
            relay_mode=RelayMode.disabled(),
        )
    )
    server_addr = server.addr()

    async def serve():
        incoming = await server.accept_next()
        conn = await (await incoming.accept()).connect()
        recv = await conn.accept_uni()
        msg = await recv.read_to_end(32)
        assert msg == b"unidirectional"

    server_task = asyncio.create_task(serve())

    client = await Endpoint.bind(
        EndpointOptions(preset=preset_n0(), relay_mode=RelayMode.disabled())
    )
    conn = await client.connect(server_addr, ALPN)
    send = await conn.open_uni()
    await send.write_all(b"unidirectional")
    await send.finish()

    await server_task
    await client.close()
    await server.close()
