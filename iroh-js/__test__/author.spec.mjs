import test from 'ava'

import { Iroh, AuthorId } from '../index.js'


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

test('create author id', (t) => {
  const authorStr = 'mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q'
  const author = AuthorId.fromString(authorStr)
  t.is(author.toString(), authorStr)

  const author0 = AuthorId.fromString(authorStr)
  t.truthy(author.isEqual(author0))
})
