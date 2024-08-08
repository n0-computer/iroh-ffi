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
  const node = await Iroh.memory()
  const client = node.node()
  const status = await client.status()

  t.is(status.version, '0.22.0')
})
