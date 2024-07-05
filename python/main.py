import iroh

import argparse
import asyncio
import os

IROH_DATA_DIR = "./iroh-data"

async def main():
    # parse arguments
    parser = argparse.ArgumentParser(description='Python Iroh Node Demo')
    parser.add_argument('--ticket', type=str, help='ticket to join a document')

    args = parser.parse_args()

    if not args.ticket:
        print("In example mode")
        print("(To run the sync demo, please provide a ticket to join a document)")
        print()

        print("creating data directory at ./iroh-data")

        # create iroh data dir if it does not exists
        if not os.path.exists(IROH_DATA_DIR):
            os.mkdir(IROH_DATA_DIR)

        # create iroh node
        node = await iroh.IrohNode.create(IROH_DATA_DIR)
        node_id = node.node_id()
        print("Started Iroh node: {}".format(node_id))

        # create doc
        doc = await node.doc_create()
        doc_id = doc.id()
        print("Created doc: {}".format(doc_id))

        doc = await node.doc_create()
        doc_id = doc.id()
        print("Created doc: {}".format(doc_id))

        # list docs
        docs = await node.doc_list()
        print("List all {} docs:".format(len(docs)))
        for doc in docs:
            print("\t{}".format(doc))

        exit()

    # create iroh data dir if it does not exists
    if not os.path.exists(IROH_DATA_DIR):
        os.mkdir(IROH_DATA_DIR)

    # create iroh node
    node = await iroh.IrohNode.create(IROH_DATA_DIR)
    node_id = node.node_id()
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
