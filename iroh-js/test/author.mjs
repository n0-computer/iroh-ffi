import { test, suite } from 'node:test'
import assert from 'node:assert'

import { Iroh, AuthorId } from '../index.js'

suite('author', () => {
  test('has a default author', async (t) => {
    const node = await Iroh.memory({ enableDocs: true })

    const defaultAuthor = await node.authors.default()
    assert.ok(defaultAuthor)
  })

  test('list authors', async (t) => {
    const node = await Iroh.memory({ enableDocs: true })

    // create an author
    await node.authors.create()

    const authors = await node.authors.list()
    assert.equal(authors.length, 2)
  })

  test('import export author', async (t) => {
    const node = await Iroh.memory({ enableDocs: true })

    // create an author
    const author = await node.authors.create()

    const fullAuthor = await node.authors.export(author)
    const authorImported = await node.authors.import(fullAuthor)

    assert.equal(author.toString(), authorImported.toString())
  })

  test('create author id', (t) => {
    const authorStr = '7db06b57aac9b3640961d281239c8f23487ac7f7265da21607c5612d3527a254'
    const author = AuthorId.fromString(authorStr)
    assert.equal(author.toString(), authorStr)

    const author0 = AuthorId.fromString(authorStr)
    assert.ok(author.isEqual(author0))
  })
})
