import { test, suite } from 'node:test'
import assert from 'node:assert'

import pkg from '../index.js'
const { RelayMap, RelayMode } = pkg

suite('relay map', () => {
  test('crud', () => {
    const map = RelayMap.empty()
    assert.ok(map.isEmpty())

    const cfg = {
      url: 'https://relay.example.org/',
      quicPort: 7842,
      authToken: 'hunter2',
    }
    map.insert(cfg)
    assert.equal(map.len(), 1)
    assert.ok(map.contains('https://relay.example.org/'))

    const got = map.get('https://relay.example.org/')
    assert.equal(got.url, cfg.url)
    assert.equal(got.quicPort, cfg.quicPort)
    assert.equal(got.authToken, cfg.authToken)

    assert.ok(map.remove('https://relay.example.org/'))
    assert.ok(map.isEmpty())
  })
})

suite('relay mode', () => {
  test('constructors', () => {
    RelayMode.disabled()
    RelayMode.defaultMode()
    RelayMode.staging()
    const map = RelayMap.fromUrls(['https://r1.example.org/'])
    RelayMode.custom(map)
    RelayMode.customFromUrls(['https://r2.example.org/'])
  })
})
