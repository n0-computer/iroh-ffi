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

    # if not args.ticket:
    #     print("In example mode")
    #     print("(To run the sync demo, please provide a ticket to join a document)")
    #     print()
    #
    #     # create iroh node
    #     options = iroh.NodeOptions()
    #     options.enable_docs = True
    #     node = await iroh.Iroh.memory_with_options(options)
    #     # FIXME: Get Node ID
    #     print(f"Started Iroh node")
    #
    #     # create doc
    #     doc = await node.docs().create()
    #     doc_id = doc.id()
    #     print("Created doc: {}".format(doc_id))
    #
    #     doc = await node.docs().create()
    #     doc_id = doc.id()
    #     print("Created doc: {}".format(doc_id))
    #
    #     # list docs
    #     docs = await node.docs().list()
    #     print("List all {} docs:".format(len(docs)))
    #     for doc in docs:
    #         print("\t{}".format(doc))
    #
    #     exit()

    # create iroh node
    node = await iroh.Iroh.memory()
    node_id = await node.node().node_id()
    print("Started Iroh node: {}".format(node_id))

    # join doc
    doc = await node.doc_join(args.ticket)
    doc_id = doc.id()
    print("Joined doc: {}".format(doc_id))

    # sync & print
    print("Waiting 5 seconds to let stuff sync...")
    await asyncio.sleep(5)
    keys = await doc.keys()
    print("Data:")
    for key in keys:
        content = await doc.get_content_bytes(key)
        print("{} : {} (hash: {})".format(key.key(), content.decode("utf8"), key.hash().to_string()))


if __name__ == "__main__":
    asyncio.run(main())
