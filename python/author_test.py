# tests that correspond to the `src/author.rs` rust api
from iroh import IrohNode
import pytest
import tempfile

def test_author_api():
    #
    # create node
    dir = tempfile.TemporaryDirectory()
    node = IrohNode(dir.name)
    #
    # creating a node also creates an author
    assert len(node.author_list()) == 1
    #
    # create
    author_id = node.author_create()
    #
    # list all authors on the node
    authors = node.author_list()
    assert len(authors) == 2
    #
    # export the author
    author = node.author_export(author_id)
    assert author_id.equal(author.id())
    #
    # remove that author from the node
    node.author_delete(author_id)
    #
    # check there are 1 authors on the node
    authors = node.author_list()
    assert len(authors) == 1
    #
    # import the author back into the node
    node.author_import(author)
    #
    # check there is 1 author on the node
    authors = node.author_list()
    assert len(authors) == 2
