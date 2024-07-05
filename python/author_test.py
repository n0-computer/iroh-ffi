# tests that correspond to the `src/author.rs` rust api
from iroh import IrohNode
import pytest
import tempfile

@pytest.mark.asyncio
async def test_author_api():
    #
    # create node
    dir = tempfile.TemporaryDirectory()
    node = await IrohNode.create(dir.name)
    #
    # creating a node also creates an author
    assert len(await node.author_list()) == 1
    #
    # create
    author_id = await node.author_create()
    #
    # list all authors on the node
    authors = await node.author_list()
    assert len(authors) == 2
    #
    # export the author
    author = await node.author_export(author_id)
    assert author_id.equal(author.id())
    #
    # remove that author from the node
    await node.author_delete(author_id)
    #
    # check there are 1 authors on the node
    authors = await node.author_list()
    assert len(authors) == 1
    #
    # import the author back into the node
    await node.author_import(author)
    #
    # check there is 1 author on the node
    authors = await node.author_list()
    assert len(authors) == 2
