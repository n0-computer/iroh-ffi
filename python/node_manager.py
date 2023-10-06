import iroh
import os

IROH_DATA_DIR = "./iroh-data"

class NodeManager:

    def __init__(self):
        # create iroh data dir if it does not exists
        if not os.path.exists(IROH_DATA_DIR):
            os.mkdir(IROH_DATA_DIR)

        # create iroh node
        self.node = iroh.IrohNode(IROH_DATA_DIR)
        self.node_id = self.node.node_id()        
        self.init_author()
        
    def init_author(self):
        self.author = self.node.author_new()

    def doc_join(self, ticket):
        doc_ticket = iroh.DocTicket.from_string(ticket)
        self.doc = self.node.doc_join(doc_ticket)
    
    def keys(self):
        return self.doc.keys()