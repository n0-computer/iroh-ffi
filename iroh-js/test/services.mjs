import { test, suite } from 'node:test'
import assert from 'node:assert'

import pkg from '../index.js'
const { Endpoint, ServicesClient, presetIrohServices, presetMinimal } = pkg

// Well-formed (but fake) API secret — the remote does not exist, but the
// client connects lazily so construction still succeeds. Validates the
// options -> builder -> client plumbing without network.
const FAKE_API_SECRET =
  'servicesaaqaobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqob' +
  '75c4sdqwvay5nwj63yzvqc7iozsh66x53lcpcy5vyc5ledl2pwdaaa'

async function endpoint() {
  const b = Endpoint.builder()
  presetMinimal(b)
  return await b.bind()
}

suite('services client', () => {
  test('boots with fake secret', async () => {
    const ep = await endpoint()
    const client = await ServicesClient.create(ep, { apiSecret: FAKE_API_SECRET })
    assert.ok(client)
    await ep.close()
  })

  test('rejects no credentials', async () => {
    const ep = await endpoint()
    await assert.rejects(ServicesClient.create(ep, {}))
    await ep.close()
  })

  test('rejects two credentials', async () => {
    const ep = await endpoint()
    await assert.rejects(
      ServicesClient.create(ep, { apiSecret: FAKE_API_SECRET, apiSecretFromEnv: true }),
    )
    await ep.close()
  })

  test('rejects malformed secret', async () => {
    const ep = await endpoint()
    await assert.rejects(ServicesClient.create(ep, { apiSecret: 'not-a-valid-ticket' }))
    await ep.close()
  })
})

suite('services preset', () => {
  const relays = ['https://relay.example.org/']

  test('binds with custom relays', async () => {
    // The relay is unreachable, but bind does not connect, so this checks the
    // mint -> relay map -> builder plumbing.
    const b = Endpoint.builder()
    presetIrohServices(b, { relays, apiSecret: FAKE_API_SECRET })
    const ep = await b.bind()
    assert.ok(ep.boundSockets().length > 0)
    await ep.close()
  })

  test('rejects bad options', () => {
    const b = () => Endpoint.builder()
    assert.throws(() => presetIrohServices(b(), { relays }))
    assert.throws(() =>
      presetIrohServices(b(), { relays, apiSecret: FAKE_API_SECRET, apiSecretFromEnv: true }),
    )
    assert.throws(() => presetIrohServices(b(), { relays, apiSecret: 'not-a-valid-ticket' }))
    assert.throws(() =>
      presetIrohServices(b(), {
        relays,
        apiSecret: FAKE_API_SECRET,
        endpointSecretKey: Buffer.alloc(16),
      }),
    )
  })

  test('pins the endpoint key', () => {
    // The token is scoped to the preset's key, so setting one afterwards must
    // throw rather than break relay auth at connect time.
    const b = Endpoint.builder()
    presetIrohServices(b, { relays, apiSecret: FAKE_API_SECRET })
    assert.throws(() => b.secretKey(Buffer.alloc(32, 7)))
  })

  test('relays', () => {
    // Omitted falls back to the n0 relays, as in Rust.
    presetIrohServices(Endpoint.builder(), { apiSecret: FAKE_API_SECRET })
    // An explicitly empty list does not.
    assert.throws(() =>
      presetIrohServices(Endpoint.builder(), { relays: [], apiSecret: FAKE_API_SECRET }),
    )
    assert.throws(() =>
      presetIrohServices(Endpoint.builder(), {
        relays: ['not a url'],
        apiSecret: FAKE_API_SECRET,
      }),
    )
  })
})
