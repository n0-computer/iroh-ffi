import { test, suite } from 'node:test'
import assert from 'node:assert'

import pkg from '../index.js'
const { Endpoint, EndpointTicket, RelayMode, presetMinimal } = pkg

const ALPN = Array.from(Buffer.from('iroh-ffi/test/0', 'utf8'))

// A "preset" in JS is any function that configures an EndpointBuilder.
async function bindMinimal() {
  const b = Endpoint.builder()
  presetMinimal(b)
  return await b.bind()
}

async function bindServer() {
  const b = Endpoint.builder()
  b.applyN0()
  b.alpns([ALPN])
  b.relayMode(RelayMode.disabled())
  return await b.bind()
}

async function bindClient() {
  const b = Endpoint.builder()
  b.applyN0()
  b.relayMode(RelayMode.disabled())
  return await b.bind()
}

suite('endpoint', () => {
  test('builder + preset: id, addr, sockets, secretKey, close', async () => {
    const ep = await bindMinimal()
    const id = ep.id()
    assert.ok(id.toString().length > 0)

    const addr = ep.addr()
    assert.ok(addr.id().equals(id))

    assert.ok(ep.boundSockets().length > 0)
    assert.deepEqual(ep.secretKey().public().toBytes(), id.toBytes())

    await ep.close()
    assert.ok(ep.isClosed())
  })

  test('builder.bind() consumes — second call errors', async () => {
    const b = Endpoint.builder()
    presetMinimal(b)
    const ep = await b.bind()
    await ep.close()
    await assert.rejects(() => b.bind(), /already consumed/)
  })

  test('custom preset function', async () => {
    // A user-defined preset is just a function over the builder.
    const myPreset = (b) => {
      b.applyMinimal()
      b.alpns([ALPN])
    }
    const b = Endpoint.builder()
    myPreset(b)
    const ep = await b.bind()
    assert.ok(ep.id().toString().length > 0)
    await ep.close()
  })

  test('endpoint ticket round trip', async () => {
    const ep = await bindMinimal()
    const addr = ep.addr()

    const ticket = EndpointTicket.fromAddr(addr)
    const str = ticket.toString()
    assert.ok(str.startsWith('endpoint'))

    const parsed = EndpointTicket.fromString(str)
    assert.ok(parsed.endpointAddr().id().equals(addr.id()))

    await ep.close()
  })

  test('endpoint ticket rejects garbage', () => {
    assert.throws(() => EndpointTicket.fromString('not-a-ticket'))
  })

  test('connect / echo / datagram round trip', async () => {
    const server = await bindServer()
    const serverAddr = server.addr()
    const serverId = server.id()

    const serverTask = (async () => {
      const incoming = await server.acceptNext()
      assert.ok(incoming)
      const accepting = await incoming.accept()
      const conn = await accepting.connect()
      assert.deepEqual(conn.alpn(), ALPN)
      const bi = await conn.acceptBi()
      const recv = bi.recv
      const send = bi.send
      const msg = await recv.readToEnd(64)
      await send.writeAll(msg)
      await send.finish()
      const dg = await conn.readDatagram()
      conn.sendDatagram(dg)
      await conn.closed()
    })()

    const client = await bindClient()
    const conn = await client.connect(serverAddr, ALPN)
    assert.ok(conn.remoteId().equals(serverId))
    assert.ok(conn.paths().length > 0)

    const bi = await conn.openBi()
    await bi.send.writeAll(Array.from(Buffer.from('hello iroh')))
    await bi.send.finish()
    const echoed = await bi.recv.readToEnd(64)
    assert.equal(Buffer.from(echoed).toString('utf8'), 'hello iroh')

    conn.sendDatagram(Array.from(Buffer.from('ping')))
    const pong = await conn.readDatagram()
    assert.equal(Buffer.from(pong).toString('utf8'), 'ping')

    const stats = conn.stats()
    assert.ok(stats.udpTxDatagrams > 0)

    conn.close(0n, Array.from(Buffer.from('bye')))
    await serverTask
    await client.close()
    await server.close()
  })

  test('unidirectional stream', async () => {
    const server = await bindServer()
    const serverAddr = server.addr()

    const serverTask = (async () => {
      const incoming = await server.acceptNext()
      const conn = await (await incoming.accept()).connect()
      const recv = await conn.acceptUni()
      const msg = await recv.readToEnd(32)
      assert.equal(Buffer.from(msg).toString('utf8'), 'unidirectional')
    })()

    const client = await bindClient()
    const conn = await client.connect(serverAddr, ALPN)
    const send = await conn.openUni()
    await send.writeAll(Array.from(Buffer.from('unidirectional')))
    await send.finish()

    await serverTask
    await client.close()
    await server.close()
  })
})
