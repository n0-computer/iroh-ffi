// tests that correspond to the `src/author.rs` rust api
import iroh.*

kotlinx.coroutines.runBlocking {
    // create node
    val node = Iroh.memory()

    // creating a node also creates an author
    assert(node.authors().list().size == 1)

    // create
    val authorId = node.authors().create()

    // list all authors on the node
    assert(node.authors().list().size == 2)

    // export the author
    val author = node.authors().export(authorId)
    assert(authorId.equal(author.id()))

    // remove that author from the node
    node.authors().delete(authorId)

    // check there are 1 authors on the node
    assert(node.authors().list().size == 1)

    // import the author back into the node
    node.authors().import(author)

    // check there is 1 author on the node
    assert(node.authors().list().size == 2)
}
