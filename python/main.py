import iroh

import argparse
import os
import time

IROH_DATA_DIR = "./iroh-data"

if __name__ == "__main__":

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
        node = iroh.IrohNode(IROH_DATA_DIR)
        print("Started Iroh node: {}".format(node.node_id()))

       # create doc
        doc = node.doc_create();
        print("Created doc: {}".format(doc.id()))
        
        doc = node.doc_create();
        print("Created doc: {}".format(doc.id()))
        
        # list docs
        docs = node.doc_list();
        print("List all {} docs:".format(len(docs)))
        for doc in docs:
            print("\t{}".format(doc))

        exit()

    # create iroh data dir if it does not exists
    if not os.path.exists(IROH_DATA_DIR):
        os.mkdir(IROH_DATA_DIR)

    # create iroh node
    node = iroh.IrohNode(IROH_DATA_DIR)
    print("Started Iroh node: {}".format(node.node_id()))

    # join doc
    doc = node.doc_join(args.ticket)
    print("Joined doc: {}".format(doc.id()))

    # sync & print
    print("Waiting 5 seconds to let stuff sync...")
    time.sleep(5)
    keys = doc.keys()
    print("Data:")
    for key in keys:
        content = doc.get_content_bytes(key)
        print("{} : {} (hash: {})".format(key.key(), content.decode("utf8"), key.hash().to_string()))
    
    
