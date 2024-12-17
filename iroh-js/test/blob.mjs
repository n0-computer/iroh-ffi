import { test, suite } from 'node:test'
import assert from 'node:assert'

import { Iroh, SetTagOption, Hash, Collection, BlobDownloadOptions } from '../index.js'
import { cwd } from 'process'
import { randomBytes } from 'crypto'

suite('blob', () => {
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
      cwd() + '/test/index.mjs',
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
    assert.equal(allDone.format, 'Raw')
    assert.ok(allDone.tag)
    assert.ok(allDone.hash)

    await node.node.shutdown()
  })

  test('hash basics', (t) => {
    const hashStr = '2kbxxbofqx5rau77wzafrj4yntjb4gn4olfpwxmv26js6dvhgjhq'
    const hexStr = 'd2837b85c585fb1053ffb64058a7986cd21e19bc72cafb5d95d7932f0ea7324f'
    const bytes = Array.from(new Uint8Array([
      0xd2, 0x83, 0x7b, 0x85, 0xc5, 0x85, 0xfb, 0x10, 0x53, 0xff, 0xb6, 0x40, 0x58, 0xa7, 0x98, 0x6c, 0xd2, 0x1e, 0x19, 0xbc, 0x72, 0xca, 0xfb, 0x5d, 0x95, 0xd7, 0x93, 0x2f, 0x0e, 0xa7, 0x32, 0x4f
    ]))

    const hash = Hash.fromString(hashStr)
    assert.equal(hash.toString(), hashStr)
    assert.equal(hash.toString('hex'), hexStr)
    assert.deepEqual(hash.toBytes(), bytes)

    const hash0 = Hash.fromBytes(bytes)
    assert.equal(hash0.toString(), hashStr)
    assert.equal(hash0.toString('hex'), hexStr)
    assert.deepEqual(hash0.toBytes(), bytes)

    assert.ok(hash.isEqual(hash0))
    assert.ok(hash0.isEqual(hash))
    assert.ok(hash.isEqual(hash))
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
    assert.equal(blobsEmpty.length, 0)

    let collection = new Collection()
    let tagsToDelete = []
    let i = 0;
    for (let blob of blobs) {
      const res = await node.blobs.addBytes(Array.from(blob))
      collection.push(`blob-${i}`, res.hash)
      tagsToDelete.push(res.hash)
      i += 1
    }

    assert.equal(collection.length(), BigInt(numFiles))
    assert.ok(!collection.isEmpty())

    const res = await node.blobs.createCollection(
      collection,
      SetTagOption.auto(),
      tagsToDelete
    )

    assert.ok(res.hash)
    assert.ok(res.tag)

    const collectionList = await node.blobs.listCollections()
    assert.equal(collectionList.length, 1)
    assert.equal(collectionList[0].hash, res.hash)
    assert.equal(collectionList[0].totalBlobsCount, BigInt(numFiles + 1))

    await node.node.shutdown()
  })

  test('share', async (t) => {
    const node = await Iroh.memory()

    const res = await node.blobs.addBytes(Array.from(Buffer.from('hello')))
    const ticket = await node.blobs.share(res.hash, res.format, 'RelayAndAddresses')

    const nodeAddr = await node.net.nodeAddr()

    assert.equal(ticket.format, res.format)
    assert.equal(ticket.hash, res.hash)
    assert.deepEqual(ticket.nodeAddr, nodeAddr)

    await node.node.shutdown()
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

    assert.ok(res.hash)
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

    assert.equal(events.length, 4)

    await node1.node.shutdown()
    await node2.node.shutdown()
  })
})
