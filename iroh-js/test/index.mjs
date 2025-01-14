import { test, suite } from 'node:test'
import assert from 'node:assert'

import { setLogLevel, keyToPath, pathToKey } from '../index.js'
import { sep } from 'node:path'

suite('helpers', () => {
  test('can set log level', (t) => {
    setLogLevel('Error')
  })

  test('pathToKey basic', (t) => {
    const path = `/foo${sep}bar`
    const key = Buffer.concat([
      Buffer.from('/foo/bar', 'utf8'),
      Buffer.from([0x00]),
    ])

    const gotKey = pathToKey(path, null, null)
    assert.deepEqual(Buffer.from(gotKey), key)

    const gotPath = keyToPath(gotKey, null, null)
    assert.equal(gotPath, path)
  })

  test('pathToKey prefix', (t) => {
    const path = `/foo${sep}bar`
    const prefix = 'prefix:'
    const key = Buffer.concat([
      Buffer.from('prefix:/foo/bar', 'utf8'),
      Buffer.from([0x00]),
    ])

    const gotKey = pathToKey(path, prefix, null)
    assert.deepEqual(Buffer.from(gotKey), key)

    const gotPath = keyToPath(gotKey, prefix, null)
    assert.equal(gotPath, path)
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
    assert.deepEqual(Buffer.from(gotKey), key)

    let gotPath = keyToPath(gotKey, prefix, root)
    assert.equal(path, gotPath)
  })
})
