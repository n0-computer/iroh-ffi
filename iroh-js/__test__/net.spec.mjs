import test from 'ava'

import { Iroh } from '../index.js'

test('node status', async (t) => {
  const iroh = await Iroh.memory()

  const nodeId = await iroh.net.nodeId()
  t.truthy(nodeId)
})