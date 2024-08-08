import test from 'ava'

import { Iroh } from '../index.js'


test('has a default author', async (t) => {
  const node = await Iroh.memory()

  const defaultAuthor = await node.authors.default()
  t.truthy(defaultAuthor)
})

test('list authors', async (t) => {
  const node = await Iroh.memory()

  // create an author
  await node.authors.create()

  const authors = await node.authors.list()
  t.is(authors.length, 2)
})

test('import export author', async (t) => {
  const node = await Iroh.memory()

  // create an author
  const author = await node.authors.create()

  const fullAuthor = await node.authors.export(author)
  const authorImported = await node.authors.import(fullAuthor)

  t.is(author.toString(), authorImported.toString())
})
