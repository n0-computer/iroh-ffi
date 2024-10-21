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

  t.is(status.version, '0.27.0')
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
  t.plan(4)
  const alpn = Buffer.from('iroh-example/hello/0')

  const protocols = {
    [alpn]: (err, ep, client) => ({
      accept: async (err, connecting) => {
        console.log('accept')
        t.falsy(err)
        const nodeId = await client.net.nodeId()
        console.log(`accepting on node ${nodeId}`)
        const alpn = await connecting.alpn()
        console.log(`incoming on ${alpn.toString()}`)

        const conn = await connecting.connect()
        const remote = await conn.getRemoteNodeId()
        console.log(`connected id ${remote.toString()}`)

        const bi = await conn.acceptBi()

        const bytes = await bi.recv.readToEnd(64)
        console.log(`got: ${bytes.toString()}`)
        t.is(bytes.toString(), 'yo')
        await bi.send.writeAll(Buffer.from('hello'))
        await bi.send.finish()
        await bi.send.stopped()
      },
      shutdown: (err) => {
        if (err != null) {
          throw err
        }
        console.log('shutting down')
      }
    })
  }
  const node1 = await Iroh.memory({
    protocols,
  })

  const nodeAddr = await node1.net.nodeAddr()

  const node2 = await Iroh.memory({ protocols })
  const status = await node2.node.status()
  console.log(`status ${status.version}`)
  const endpoint = node2.node.endpoint()
  console.log(`connecting to ${nodeAddr.nodeId}`)

  const conn = await endpoint.connect(nodeAddr, alpn)
  const remote = await conn.getRemoteNodeId()
  console.log(`connected to ${remote.toString()}`)

  const bi = await conn.openBi()

  await bi.send.writeAll(Buffer.from('yo'))
  await bi.send.finish()
  await bi.send.stopped()

  let out = Buffer.alloc(5)
  await bi.recv.readExact(out)

  console.log(`read: ${out.toString()}`)
  t.is(out.toString(), 'hello')

  await node2.node.shutdown(false)
  await node1.node.shutdown(false)

  console.log('end')
  t.pass()
})
