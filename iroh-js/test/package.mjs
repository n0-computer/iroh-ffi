import { test } from 'node:test'
import assert from 'node:assert'
import { existsSync, readFileSync } from 'node:fs'

const pkg = JSON.parse(readFileSync(new URL('../package.json', import.meta.url)))

// Consumers resolve the package through these fields, unlike the other tests
// which import ../index.js directly.
test('package.json main and types point at existing files', () => {
  for (const field of ['main', 'types']) {
    const target = new URL(`../${pkg[field]}`, import.meta.url)
    assert.ok(existsSync(target), `${field} points at missing file: ${pkg[field]}`)
  }
})
