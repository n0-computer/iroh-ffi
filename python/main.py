import time
import argparse

from node_manager import NodeManager


if __name__ == "__main__":

    # parse arguments
    parser = argparse.ArgumentParser(description='Python Iroh Node Demo')
    parser.add_argument('--ticket', type=str, help='ticket to join a document')

    args = parser.parse_args()

    if not args.ticket:
        print("Please provide a ticket to join a document")
        exit()

    node_manager = NodeManager()
    print("Iroh node id: {}".format(node_manager.node_id))
    node_manager.doc_join(args.ticket)
    print("Iroh document id: {}".format(node_manager.doc.id()))

    time.sleep(5)
    print("Iroh document keys: {}".format(node_manager.keys()))

    
    