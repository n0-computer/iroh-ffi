import test from 'ava'

import { Iroh, PublicKey, verifyNodeAddr, Query, AuthorId, pathToKey, keyToPath, DocTicket } from '../index.js'

import { tmpdir } from 'node:os'
import { randomBytes } from 'node:crypto'
import { join } from 'node:path'
import { mkdtemp, mkdir, writeFile, readFile } from 'node:fs/promises'

test('create doc', async (t) => {
  const node = await Iroh.memory()

  const doc = await node.docs.create()
  const id = doc.id()
  t.truthy(id)

  const author = await node.authors.default()
  const key = Array.from(new Uint8Array([1, 2, 3]))
  const value = Array.from(new Uint8Array([3, 2, 1]))
  const hash = await doc.setBytes(author, key, value)
  t.truthy(hash)

  const entry = await doc.getExact(author, key, false)
  t.truthy(entry)
  t.is(entry.hash, hash.toString())
  t.is(entry.author, author.toString())
  t.deepEqual(entry.key, key)
  t.is(entry.len, BigInt(key.length))
})

test('basic sync', async (t) => {
  const node0 = await Iroh.memory()
  const node1 = await Iroh.memory()

  const doc0 = await node0.docs.create()
  const ticket = await doc0.share('Write', 'RelayAndAddresses')
  const ticketString = ticket.toString()
  const ticketBack = DocTicket.fromString(ticketString)

  // Do not use Promise.withResovlers it is buggy
  let resolve0;
  let reject0;
  const promise0 = new Promise((resolve, reject) => {
    resolve0 = resolve;
    reject0 = reject;
  });
  let resolve1;
  let reject1;
  const promise1 = new Promise((resolve, reject) => {
    resolve1 = resolve;
    reject1 = reject;
  });

  await doc0.subscribe((error, event) => {
    if (error != null) {
      return reject0(error)
    }
    // Wait until the sync is finished
    if (event.contentReady != null) {
      resolve0(event.contentReady.hash)
    }
  })

  const doc1 = await node1.docs.joinAndSubscribe(ticket, (error, event) => {
    if (error != null) {
      return reject1(error)
    }
    // Wait until the sync is finished
    if (event.syncFinished != null) {
      resolve1(event)
    }
  })

  const e = await promise1

  // create content on node1
  const author = await node1.authors.default()
  await doc1.setBytes(
    author,
    Array.from(Buffer.from('hello')),
    Array.from(Buffer.from('world'))
  )


  const hash = await promise0
  const val = await node1.blobs.readToBytes(hash)
  t.is(Buffer.from(val).toString('utf8'), 'world')
})


test('node addr', (t) => {
  const keyStr = 'ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva'
  const nodeId = PublicKey.fromString(keyStr)

  const ipv4 = '127.0.0.1:3000'
  const ipv6 = '::1:3000'
  const addrs = [ipv4, ipv6]

  const relayUrl = 'https://example.com'

  const nodeAddr = { nodeId: nodeId.toString(), relayUrl, addrs }
  verifyNodeAddr(nodeAddr)

  t.pass()
})

test('query', async (t) => {
  const query1 = Query.singleLatestPerKey({
    direction: 'Desc',
  })

  t.is(query1.offset(), BigInt(0))
  t.is(query1.limit(), null)

  const query2 = Query.author(
    AuthorId.fromString('mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q'),
    {
      direction: 'Asc',
      offset: BigInt(100),
    }
  )
  t.is(query2.offset(), BigInt(100))
  t.is(query2.limit(), null)

  const query3 = Query.keyExact(
    Array.from(Buffer.from('key')),
    {
      sortBy: 'KeyAuthor',
      offset: BigInt(0),
      limit: BigInt(100),
    }
  )
  t.is(query3.offset(), BigInt(0))
  t.is(query3.limit(), BigInt(100))

  const query4 = Query.keyPrefix(
    Array.from(Buffer.from('prefix')),
    {
      limit: BigInt(100)
    }
  )
  t.is(query3.offset(), BigInt(0))
  t.is(query3.limit(), BigInt(100))
})


test('import export', async (t) => {
  const dir = await mkdtemp(join(tmpdir(), 'iroh-docs-'));
  const inRoot = join(dir, 'in')
  const outRoot = join(dir, 'out')
  await mkdir(inRoot)
  await mkdir(outRoot)

  const filePath = join(inRoot, 'test')
  const size = 100
  const bytes = randomBytes(size)
  await writeFile(filePath, bytes)

  const node = await Iroh.memory()
  const author = await node.authors.default()
  const doc = await node.docs.create()

  // import entry
  const key = pathToKey(filePath, null, inRoot)
  await doc.importFile(author, key, filePath, true)

  // get entry
  const query = Query.authorKeyExact(author, key)
  const entry = await doc.getOne(query)

  // export entry
  const exportPath = keyToPath(key, null, outRoot)
  await doc.exportFile(entry, exportPath)

  // read file
  const content = await readFile(exportPath)

  t.deepEqual(content, bytes)
})
