// tests that correspond to the `src/author.rs` rust api
import iroh.*

kotlinx.coroutines.runBlocking {
    // create node
    val dir = kotlin.io.path.createTempDirectory("author-test")
    val node = IrohNode.create(dir.toString())

    // creating a node also creates an author
    assert(node.authorList().size == 1)

    // create
    val authorId = node.authorCreate()

    // list all authors on the node
    assert(node.authorList().size == 2)

    // export the author
    val author = node.authorExport(authorId)
    assert(authorId.equal(author.id()))

    // remove that author from the node
    node.authorDelete(authorId)

    // check there are 1 authors on the node
    assert(node.authorList().size == 1)

    // import the author back into the node
    node.authorImport(author)

    // check there is 1 author on the node
    assert(node.authorList().size == 2)
}
