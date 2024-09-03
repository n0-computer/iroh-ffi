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
