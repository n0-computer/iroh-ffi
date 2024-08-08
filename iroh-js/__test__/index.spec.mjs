import test from 'ava'

import { sum, setLogLevel, keyToPath, pathToKey } from '../index.js'

test('sum from native', (t) => {
  t.is(sum(1, 2), 3)
})

test('can set log level', (t) => {
  setLogLevel('Error')
  t.pass()
})

test('pathToKey basic', (t) => {
  const path = '/foo/bar'
  const key = Buffer.concat([
    Buffer.from('/foo/bar', 'utf8'),
    Buffer.from([0x00]),
  ])

  const gotKey = pathToKey(path, null, null)
  t.deepEqual(Buffer.from(gotKey), key)

  const gotPath = keyToPath(gotKey, null, null)
  t.is(gotPath, path)
})

test('pathToKey prefix', (t) => {
  const path = '/foo/bar'
  const prefix = 'prefix:'
  const key = Buffer.concat([
    Buffer.from('prefix:/foo/bar', 'utf8'),
    Buffer.from([0x00]),
  ])

  const gotKey = pathToKey(path, prefix, null)
  t.deepEqual(Buffer.from(gotKey), key)

  const gotPath = keyToPath(gotKey, prefix, null)
  t.is(gotPath, path)
})

test('pathToKey root', (t) => {
  let path = '/foo/bar'
  let prefix = 'prefix:'
  let root = "/foo"

  const key = Buffer.concat([
    Buffer.from('prefix:bar', 'utf8'),
    Buffer.from([0x00]),
  ])

  const gotKey = pathToKey(path, prefix, root)
  t.deepEqual(Buffer.from(gotKey), key)

  let gotPath = keyToPath(gotKey, prefix, root)
  t.is(path, gotPath)
})
