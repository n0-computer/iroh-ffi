from signal import pause
import iroh

import argparse
import asyncio

async def main():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    # parse arguments
    parser = argparse.ArgumentParser(description='Python Iroh Node Demo')
    parser.add_argument('--ticket', type=str, help='ticket to join a document')

    args = parser.parse_args()

    # create iroh node
    options = iroh.NodeOptions()
    options.enable_docs = True
    node = await iroh.Iroh.memory_with_options(options)
    node_id = await node.net().node_id()
    print("Started Iroh node: {}".format(node_id))

    if not args.ticket:
        print("In example mode")
        print("(To run the sync demo, please provide a ticket to join a document)")
        print()

        # create doc
        doc = await node.docs().create()
        author = await node.authors().create()
        doc_id = doc.id()
        # create ticket to share doc
        ticket = await doc.share(iroh.ShareMode.READ, iroh.AddrInfoOptions.RELAY_AND_ADDRESSES)

        # add data to doc
        await doc.set_bytes(author, b"hello", b"world")
        await doc.set_bytes(author, b"foo", b"bar")
        await doc.set_bytes(author, b"baz", b"qux")

        print("Created doc: {}".format(doc_id))
        print("Keep this running and in another terminal run:\n\npython main.py --ticket {}".format(ticket))
    else:
        # join doc
        doc_ticket = iroh.DocTicket(args.ticket)
        doc = await node.docs().join(doc_ticket)
        doc_id = doc.id()
        print("Joined doc: {}".format(doc_id))

        # sync & print
        print("Waiting 5 seconds to let stuff sync...")
        await asyncio.sleep(5)

        # query all keys
        query = iroh.Query.all(None)
        keys = await doc.get_many(query)

        print("Data:")
        for entry in keys:
            # get key, hash, and content for each entry
            key = entry.key()
            hash = entry.content_hash()
            content = await entry.content_bytes(doc)
            print("{} : {} (hash: {})".format(key.decode("utf8"), content.decode("utf8"), hash))

    input("Press Enter to exit...")

if __name__ == "__main__":
    asyncio.run(main())
