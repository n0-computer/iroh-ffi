import test from 'ava'

import { Iroh, SetTagOption, Hash, Collection, BlobDownloadOptions } from '../index.js'
import { cwd } from 'process'
import { randomBytes } from 'crypto'


test('add blob from path', async (t) => {
  const node = await Iroh.memory()

  // Do not use Promise.withResovlers it is buggy
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res
    reject = rej;
  });

  await node.blobs.addFromPath(
    cwd() + '/__test__/index.spec.mjs',
    true,
    SetTagOption.auto(),
    { wrap: false },
    (error, progress) => {
      if (error != null) {
        return reject(error)
      }

      // console.log('progress', progress)
      if (progress.allDone != null) {
        resolve(progress.allDone)
      }
    }
  )

  const allDone = await promise
  t.is(allDone.format, 'Raw')
  t.truthy(allDone.tag)
  t.truthy(allDone.hash)
})

test('hash basics', (t) => {
  const hashStr = '2kbxxbofqx5rau77wzafrj4yntjb4gn4olfpwxmv26js6dvhgjhq'
  const hexStr = 'd2837b85c585fb1053ffb64058a7986cd21e19bc72cafb5d95d7932f0ea7324f'
  const bytes = Array.from(new Uint8Array([
    0xd2, 0x83, 0x7b, 0x85, 0xc5, 0x85, 0xfb, 0x10, 0x53, 0xff, 0xb6, 0x40, 0x58, 0xa7, 0x98, 0x6c, 0xd2, 0x1e, 0x19, 0xbc, 0x72, 0xca, 0xfb, 0x5d, 0x95, 0xd7, 0x93, 0x2f, 0x0e, 0xa7, 0x32, 0x4f
  ]))

  const hash = Hash.fromString(hashStr)
  t.is(hash.toString(), hashStr)
  t.is(hash.toString('hex'), hexStr)
  t.deepEqual(hash.toBytes(), bytes)

  const hash0 = Hash.fromBytes(bytes)
  t.is(hash0.toString(), hashStr)
  t.is(hash0.toString('hex'), hexStr)
  t.deepEqual(hash0.toBytes(), bytes)

  t.truthy(hash.isEqual(hash0))
  t.truthy(hash0.isEqual(hash))
  t.truthy(hash.isEqual(hash))
})

test('collections', async (t) => {
  const numFiles = 3
  const blobSize = 100

  const blobs = []
  for (let i = 0; i < numFiles; i += 1) {
    const buf = randomBytes(blobSize)
    blobs.push(buf)
  }

  const node = await Iroh.memory()
  const blobsEmpty = await node.blobs.list()
  t.is(blobsEmpty.length, 0)

  let collection = new Collection()
  let tagsToDelete = []
  let i = 0;
  for (let blob of blobs) {
    const res = await node.blobs.addBytes(Array.from(blob))
    collection.push(`blob-${i}`, res.hash)
    tagsToDelete.push(res.hash)
    i += 1
  }

  t.is(collection.length(), BigInt(numFiles))
  t.falsy(collection.isEmpty())

  const res = await node.blobs.createCollection(
    collection,
    SetTagOption.auto(),
    tagsToDelete
  )

  t.truthy(res.hash)
  t.truthy(res.tag)

  const collectionList = await node.blobs.listCollections()
  t.is(collectionList.length, 1)
  t.is(collectionList[0].hash, res.hash)
  t.is(collectionList[0].totalBlobsCount, BigInt(numFiles + 1))
})

test('share', async (t) => {
  const node = await Iroh.memory()

  const res = await node.blobs.addBytes(Array.from(Buffer.from('hello')))
  const ticket = await node.blobs.share(res.hash, res.format, 'RelayAndAddresses')

  const nodeAddr = await node.net.nodeAddr()

  t.is(ticket.format, res.format)
  t.is(ticket.hash, res.hash)
  t.deepEqual(ticket.nodeAddr, nodeAddr)
})

test('provide events', async (t) => {
  const node1 = await Iroh.memory()

  // Do not use Promise.withResovlers it is buggy
  let resolve0
  let reject0
  const promise0 = new Promise((res, rej) => {
    resolve0 = res
    reject0 = rej
  })

  let resolve1
  let reject1
  const promise1 = new Promise((res, rej) => {
    resolve1 = res
    reject1 = rej
  })

  let events = []
  const node2 = await Iroh.memory({ blobEvents: (err, event) => {
    if (err != null) {
      return reject0(err)
    }

    events.push(event)

    if (event.transferCompleted != null) {
      return resolve0()
    }
  }})

  const res = await node2.blobs.addBytes(Array.from(Buffer.from('hello')))

  t.truthy(res.hash)
  const node2Addr = await node2.net.nodeAddr()

  const opts = new BlobDownloadOptions(res.format, [node2Addr], SetTagOption.auto())
  await node1.blobs.download(res.hash, opts, (err, event) => {
    if (err != null) {
      return reject1(err)
    }

    if (event.allDone != null) {
      return resolve1(event)
    }
  })

  await promise0
  await promise1

  t.is(events.length, 4)
})
