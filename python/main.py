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
        print("Please provide a ticket to join a document")
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

    
    