import { test, suite } from 'node:test'
import assert from 'node:assert'

import pkg from '../index.js'
const { EndpointId, SecretKey, Signature } = pkg

const keyStr = '523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea'
const fmtStr = '523c7996ba'
const bytes = Array.from(new Uint8Array([
  0x52, 0x3c, 0x79, 0x96, 0xba, 0xd7, 0x74, 0x24,
  0xe9, 0x67, 0x86, 0xcf, 0x7a, 0x72, 0x05, 0x11,
  0x53, 0x37, 0xa5, 0xb4, 0x56, 0x5c, 0xd2, 0x55,
  0x06, 0xa0, 0xf2, 0x97, 0xb1, 0x91, 0xa5, 0xea,
]))

suite('endpoint id', () => {
  test('from string', () => {
    const id = EndpointId.fromString(keyStr)
    assert.equal(id.toString(), keyStr)
    assert.deepEqual(id.toBytes(), bytes)
    assert.equal(id.fmtShort(), fmtStr)
  })

  test('from bytes', () => {
    const id = EndpointId.fromBytes(bytes)
    assert.equal(id.toString(), keyStr)
    assert.deepEqual(id.toBytes(), bytes)
    assert.equal(id.fmtShort(), fmtStr)
  })

  test('equality', () => {
    assert.ok(EndpointId.fromString(keyStr).equals(EndpointId.fromBytes(bytes)))
  })

  test('rejects bad bytes', () => {
    assert.throws(() => EndpointId.fromBytes([1, 2, 3]))
  })
})

suite('secret key', () => {
  test('bytes round trip', () => {
    const secret = SecretKey.generate()
    const raw = secret.toBytes()
    assert.equal(raw.length, 32)
    const secret2 = SecretKey.fromBytes(raw)
    assert.deepEqual(secret.toBytes(), secret2.toBytes())
    assert.deepEqual(secret.public().toBytes(), secret2.public().toBytes())
  })

  test('sign / verify round trip', () => {
    const secret = SecretKey.generate()
    const pub = secret.public()
    const msg = Array.from(Buffer.from('hello iroh', 'utf8'))
    const sig = secret.sign(msg)

    const raw = sig.toBytes()
    assert.equal(raw.length, 64)
    const sig2 = Signature.fromBytes(raw)
    assert.deepEqual(sig2.toBytes(), raw)

    pub.verify(msg, sig)
    pub.verify(msg, sig2)
  })

  test('verify rejects tampered message', () => {
    const secret = SecretKey.generate()
    const pub = secret.public()
    const sig = secret.sign(Array.from(Buffer.from('original')))
    assert.throws(() => pub.verify(Array.from(Buffer.from('tampered')), sig))
  })
})
