# tests that correspond to the `src/author.rs` rust api
from iroh import Iroh, NodeOptions
import pytest
import iroh
import asyncio

@pytest.mark.asyncio
async def test_author_api():
    # setup event loop, to ensure async callbacks work
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())

    #
    # create node
    options = NodeOptions()
    options.enable_docs = True
    node = await Iroh.memory_with_options(options)
    #
    # creating a node also creates an author
    assert len(await node.authors().list()) == 1
    #
    # create
    author_id = await node.authors().create()
    #
    # list all authors on the node
    authors = await node.authors().list()
    assert len(authors) == 2
    #
    # export the author
    author = await node.authors().export(author_id)
    assert author_id.equal(author.id())
    #
    # remove that author from the node
    await node.authors().delete(author_id)
    #
    # check there are 1 authors on the node
    authors = await node.authors().list()
    assert len(authors) == 1
    #
    # import the author back into the node
    await node.authors().import_author(author)
    #
    # check there is 1 author on the node
    authors = await node.authors().list()
    assert len(authors) == 2
