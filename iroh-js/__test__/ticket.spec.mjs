import test from 'ava'

import { Iroh, Hash, BlobTicket } from '../index.js'


test('ticket creation and encoding', async (t) => {
  const node = await Iroh.memory()
  const nodeId = await node.net.nodeId()

  const hash1 = new Hash(Array.from(new Uint8Array([1, 2, 3])))
  const hash2 = new Hash(Array.from(new Uint8Array([1, 2, 3, 4])))

  const ticket1 = new BlobTicket(
    {
      nodeId: nodeId.toString(),
    },
    hash1.toString(),
    'Raw',
  )
  const ticket2 = new BlobTicket(
    {
      nodeId: nodeId.toString(),
    },
    hash2.toString(),
    'Raw',
  )


  const ticketString1 = ticket1.toString()
  t.truthy(ticketString1)
  const ticketString2 = ticket2.toString()
  t.truthy(ticketString2)

  const ticketBack1 = BlobTicket.fromString(ticketString1)
  const ticketBack2 = BlobTicket.fromString(ticketString2)

  t.truthy(ticketBack1.isEqual(ticket1))
  t.truthy(ticketBack2.isEqual(ticket2))
  t.falsy(ticketBack1.isEqual(ticketBack2))
  t.falsy(ticket1.isEqual(ticket2))
})
