// SPIKE: an Endpoint created via the core addon, used by the separately-built ping addon.
import { createRequire } from 'node:module'
const require = createRequire(import.meta.url)

const core = require('./iroh/js_iroh_core.node')
const ping = require('./iroh_ping/js_iroh_ping.node')

console.log('  core addon exports:', Object.keys(core).sort().join(', '))
console.log('  ping addon exports:', Object.keys(ping).sort().join(', '))

// Endpoint comes from the shared dylib, surfaced through the CORE addon.
const server = await core.Endpoint.bind()
const client = await core.Endpoint.bind()
console.log('  server bound:', server.addr().toString())
console.log('  client bound:', client.addr().toString())

// Ping comes from the PLUGIN addon and consumes the core addon's Endpoint.
const p = new ping.Ping()
await p.serve(server)
const rtt = await p.ping(client, server.addr())
console.log(`  PING -> PONG round trip: ${rtt} ms`)

await client.close()
await server.close()
console.log('  OK: Endpoint crossed the addon boundary')
