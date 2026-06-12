import { test, suite } from 'node:test'
import assert from 'node:assert'

import pkg from '../index.js'
const { ClientHost, Endpoint, ServicesClient, clientHostAlpn, presetMinimal } = pkg

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

  test('remote diagnostics boots with fake secret', async () => {
    const ep = await endpoint()
    const client = await ServicesClient.create(ep, {
      apiSecret: FAKE_API_SECRET,
      remoteDiagnostics: true,
    })
    assert.ok(client)
    await ep.close()
  })

  test('remote diagnostics rejects ssh key credential', async () => {
    const ep = await endpoint()
    // Match the message: a malformed pem also rejects, and this test must
    // fail if the remoteDiagnostics guard (not pem parsing) goes.
    await assert.rejects(
      ServicesClient.create(ep, { sshKeyPem: 'irrelevant', remoteDiagnostics: true }),
      /remoteDiagnostics/,
    )
    await ep.close()
  })

  test('client host constructs and alpn is stable', async () => {
    const ep = await endpoint()
    const host = new ClientHost(ep)
    assert.ok(host)
    assert.equal(Buffer.from(clientHostAlpn()).toString(), 'n0/n0des-client-host/1')
    await ep.close()
  })
})
