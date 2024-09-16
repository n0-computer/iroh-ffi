import test from 'ava'

import { Iroh } from '../index.js'


test('gossip basic', async (t) => {
  const n0 = await Iroh.memory()
  const n1 = await Iroh.memory()

  let rawTopic = new Uint8Array(32)
  rawTopic.fill(0, 1, 32)
  const topic = Array.from(rawTopic)

  const n1Id = await n1.net.nodeId()
  const n1Addr = await n1.net.nodeAddr()
  await n0.net.addNodeAddr(n1Addr)

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

  const sink0 = await n0.gossip.subscribe(topic, [n1Id], (error, event) => {
    if (error != null) {
      return reject0(error)
    }

    if (event.joined != null) {
      resolve0(event)
    }
  })

  const n0Id = await n0.net.nodeId()
  const n0Addr = await n0.net.nodeAddr()
  await n1.net.addNodeAddr(n0Addr)

  const sink1 = await n1.gossip.subscribe(topic, [n0Id], (error, event) => {
    if (error != null) {
      return reject1(error)
    }

    if (event.received != null) {
      resolve1(event.received)
    }
  })

  await promise0

  // wait for n1 to show up for n0

  const msg = Array.from(Buffer.from('hello', 'utf8'))
  await sink0.broadcast(msg)

  // wait for node1 to receive the message
  const m = await promise1
  t.is(n0Id.toString(), m.deliveredFrom)
  t.is(Buffer.from(m.content).toString(), 'hello')

  await sink0.close()
  await sink1.close()

  await n0.node.shutdown(false)
  await n1.node.shutdown(false)

  t.pass()
})
