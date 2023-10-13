import iroh

import argparse
import os
import time

IROH_DATA_DIR = "./iroh-data"

if __name__ == "__main__":

    # parse arguments
    parser = argparse.ArgumentParser(description='Python Iroh Node Demo')
    parser.add_argument('--ticket', type=str, help='ticket to join a document')
    parser.add_argument('--path', type=str, help='path to add to the blob store')

    args = parser.parse_args()

    if args.path:
        # create iroh node
        node = iroh.IrohNode(IROH_DATA_DIR)
        print("Started Iroh node: {}".format(node.node_id()))

        print("Adding {} to the blob store...".format(args.path))
        blobs = node.blob_add(args.path, False, None, False, None)
        
        for blob in blobs:
            print("hash {}, name {}, size {}".format(blob.hash.to_string(), blob.name, blob.size))

        print("\nCollection information:")
        collections = node.blob_list_collections()
        for collection in collections:
            print("hash: {}, tag: {}, count: {}, size: {}".format(collection.hash.to_string(), collection.tag, collection.total_blobs_count, collection.total_blobs_size))

        exit()

    if not args.ticket:
        print("In example mode")
        print("(To run the sync demo, please provide a ticket to join a document)")
        print()

        print("creating data directory at ./iroh-data")

        # create iroh data dir if it does not exists
        if not os.path.exists(IROH_DATA_DIR):
            os.mkdir(IROH_DATA_DIR)

        # create iroh node
        node = iroh.IrohNode(IROH_DATA_DIR)
        print("Started Iroh node: {}".format(node.node_id()))

       # create doc
        doc = node.doc_new();
        print("Created doc: {}".format(doc.id()))
        
        doc = node.doc_new();
        print("Created doc: {}".format(doc.id()))
        
        # list docs
        docs = node.doc_list();
        print("List all {} docs:".format(len(docs)))
        for doc in docs:
            print("\t{}".format(doc.to_string()))

        # ensure blob_list_incomplete executes
        blobs = node.blob_list_incomplete()
        if len(blobs) != 0:
            print("Unexpected incomplete blobs:")
            for blob in blobs:
                print("\thash: {} expected_size: {} size: {}".format(blob.hash.to_string(), blob.expected_size, blob.size))

        # ensure blob_validate executes
        # TODO: unimplemented in v0.7.0
        # blobs = node.blob_validate(False)
        # if len(blobs) != 0:
        #     print("Unexpected invalid blobs:")
        #     for blob in blobs:
        #         print("\tname: {} hash: {} size: {}".format(blob.name, blob.hash.to_string(), blob.size))

        exit()

    # create iroh data dir if it does not exists
    if not os.path.exists(IROH_DATA_DIR):
        os.mkdir(IROH_DATA_DIR)

    # create iroh node
    node = iroh.IrohNode(IROH_DATA_DIR)
    print("Started Iroh node: {}".format(node.node_id()))

    # join doc
    doc_ticket = iroh.DocTicket.from_string(args.ticket)
    doc = node.doc_join(doc_ticket)
    print("Joined doc: {}".format(doc.id()))

    # sync & print
    print("Waiting 5 seconds to let stuff sync...")
    time.sleep(5)
    keys = doc.keys()
    print("Data:")
    for key in keys:
        content = doc.get_content_bytes(key)
        print("{} : {} (hash: {})".format(key.key(), content.decode("utf8"), key.hash().to_string()))
    
    
