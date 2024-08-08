import test from 'ava'

import { Iroh } from '../index.js'


test('create doc', async (t) => {
  const node = await Iroh.memory()

  const doc = await node.docs.create()
  const id = doc.id()
  t.truthy(id)

  const author = await node.authors.default()
  const key = Array.from(new Uint8Array([1, 2, 3]))
  const value = Array.from(new Uint8Array([3, 2, 1]))
  const hash = await doc.setBytes(author, key, value)
  t.truthy(hash)

  const entry = await doc.getExact(author, key, false)
  t.truthy(entry)
  t.is(entry.hash, hash.toString())
  t.is(entry.author, author.toString())
  t.deepEqual(entry.key, key)
  t.is(entry.len, BigInt(key.length))
})
