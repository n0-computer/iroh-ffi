import test from 'ava'

import { Iroh } from '../index.js'


test('create memory node', async (t) => {
  const node = await Iroh.memory()
  t.pass()
})

test('create memory node, with options', async (t) => {
  const node = await Iroh.memory({
    gcIntervalMillis: 10000
  })
  t.pass()
})

test('node status', async (t) => {
  const iroh = await Iroh.memory()
  const status = await iroh.node.status()

  t.is(status.version, '0.24.0')
})

test('rpc client memory node', async (t) => {
  const node = await Iroh.memory({
    enableRpc: true
  })

  const nodeId = await node.net.nodeId()

  const client = await Iroh.client()
  const clientId = await client.net.nodeId()

  t.is(nodeId, clientId)
})


test('custom protocol', async (t) => {
  const protocols = {
    [Buffer.from('iroh-example/text-search/0', 'utf8')]: {
      accept: async (err, connecting) => {
        if (err) {
          throw err
        }
        const alpn = await connecting.alpn()
        const alpnString = Buffer.from(alpn).toString()
        console.log(`incoming on ${alpnString}`)
        const conn = await connecting.connect()
        const remote = await conn.getRemoteNodeId()
        console.log(`connected id ${remote.toString()}`)

        const bi = await conn.acceptBi()
        const send = await bi.send()
        const recv = await bi.recv()

        const bytes = await recv.readToEnd(64)
        const b = Buffer.from(bytes)
        console.log(`got: ${b.toString()}`)
        await send.writeAll(Uint8Array.from(Buffer.from('hello')))
        await send.finish()
        await send.stopped()
      }
    }
  }
  const node1 = await Iroh.memory({
    protocols,
  })

  const nodeId = await node1.net.nodeId()

  const node2 = await Iroh.memory({ protocols })
  const status = await node2.node.status()
  console.log(`status ${status.version}`)
  const endpoint = node2.node.endpoint()
  console.log(`connecting to ${nodeId}`)
  const alpn = Array.from(Buffer.from('iroh-example/text-search/0'))
  const conn = await endpoint.connectByNodeId(nodeId, alpn)
  const remote = await conn.getRemoteNodeId()
  console.log(`connected to ${remote.toString()}`)

  const bi = await conn.openBi()
  const send = await bi.send()
  const recv = await bi.recv()

  await send.writeAll(Uint8Array.from(Buffer.from('yo')))
  await send.finish()
  await send.stopped()

  let out = Uint8Array.from(Buffer.alloc(5))
  await recv.readExact(out)

  console.log(`read: ${Buffer.from(out)}`)

  t.pass()
})
