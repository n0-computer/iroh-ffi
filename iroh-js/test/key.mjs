import { test, suite} from 'node:test'
import assert from 'node:assert'

import { PublicKey } from '../index.js'

const keyStr = '523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea'
const fmtStr = '523c7996ba'
const bytes = Array.from(new Uint8Array([0x52, 0x3c, 0x79, 0x96, 0xba, 0xd7, 0x74, 0x24, 0xe9, 0x67, 0x86, 0xcf, 0x7a, 0x72, 0x05, 0x11, 0x53, 0x37, 0xa5, 0xb4, 0x56, 0x5c, 0xd2, 0x55, 0x06, 0xa0, 0xf2, 0x97, 0xb1, 0x91, 0xa5, 0xea]))

suite('key', () => {
  test('create key from string', (t) => {
    const key = PublicKey.fromString(keyStr)
    assert.equal(key.toString(), keyStr)
    assert.deepEqual(key.toBytes(), bytes)
    assert.equal(key.fmtShort(), fmtStr)
  })

  test('create key from bytes', (t) => {
    const key = PublicKey.fromBytes(bytes)
    assert.equal(key.toString(), keyStr)
    assert.deepEqual(key.toBytes(), bytes)
    assert.equal(key.fmtShort(), fmtStr)
  })

  test('key equality', (t) => {
    const key1 = PublicKey.fromString(keyStr)
    const key2 = PublicKey.fromBytes(bytes)
    assert.ok(key1.isEqual(key2))
  })
})
