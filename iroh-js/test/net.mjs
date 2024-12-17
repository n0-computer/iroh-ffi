import { test, suite } from 'node:test'
import assert from 'node:assert'

import { Iroh } from '../index.js'

suite('net', () => {
  test('node status', async (t) => {
    const iroh = await Iroh.memory()

    const nodeId = await iroh.net.nodeId()
    assert.ok(nodeId)
  })
})
