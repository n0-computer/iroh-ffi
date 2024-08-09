import test from 'ava'

import { setLogLevel, keyToPath, pathToKey } from '../index.js'
import { sep } from 'path'

test('can set log level', (t) => {
  setLogLevel('Error')
  t.pass()
})

test('pathToKey basic', (t) => {
  const path = `${sep}foo${sep}bar`
  const key = Buffer.concat([
    Buffer.from(`${sep}foo${sep}bar`, 'utf8'),
    Buffer.from([0x00]),
  ])

  const gotKey = pathToKey(path, null, null)
  t.deepEqual(Buffer.from(gotKey), key)

  const gotPath = keyToPath(gotKey, null, null)
  t.is(gotPath, path)
})

test('pathToKey prefix', (t) => {
  const path = `${sep}foo${sep}bar`
  const prefix = 'prefix:'
  const key = Buffer.concat([
    Buffer.from(`prefix:${sep}foo${sep}bar`, 'utf8'),
    Buffer.from([0x00]),
  ])

  const gotKey = pathToKey(path, prefix, null)
  t.deepEqual(Buffer.from(gotKey), key)

  const gotPath = keyToPath(gotKey, prefix, null)
  t.is(gotPath, path)
})

test('pathToKey root', (t) => {
  let path = `${sep}foo${sep}bar`
  let prefix = 'prefix:'
  let root = `${sep}foo`

  const key = Buffer.concat([
    Buffer.from('prefix:bar', 'utf8'),
    Buffer.from([0x00]),
  ])

  const gotKey = pathToKey(path, prefix, root)
  t.deepEqual(Buffer.from(gotKey), key)

  let gotPath = keyToPath(gotKey, prefix, root)
  t.is(path, gotPath)
})
