// tests that correspond to the `src/doc.rs` rust api

import iroh.*
import kotlinx.coroutines.channels.*
import kotlinx.coroutines.runBlocking
import kotlin.random.Random

fun generateRandomByteArray(size: Int): ByteArray {
    val byteArray = ByteArray(size)
    Random.nextBytes(byteArray)
    return byteArray
}

class Subscriber : SubscribeCallback {
    val channel = Channel<Hash>(1)

    override suspend fun `event`(`event`: LiveEvent) {
        println(event.type())
        if (event.type() == LiveEventType.CONTENT_READY) {
            println("got event type content ready")
            this.channel.send(event.asContentReady())
        }
    }
}

runBlocking {
    // Create node_0
    val irohDir0 = kotlin.io.path.createTempDirectory("node-test-0")
    println(irohDir0.toString())
    val node0 = IrohNode.create(irohDir0.toString())

    // Create node_1
    val irohDir1 = kotlin.io.path.createTempDirectory("node-test-1")
    println(irohDir1.toString())
    val node1 = IrohNode.create(irohDir1.toString())

    // Create doc on node_0
    val doc0 = node0.docCreate()

    // Subscribe to sync events
    val cb = Subscriber()
    doc0.subscribe(cb)

    // Join the same doc from node_1
    val ticket = doc0.share(ShareMode.WRITE, AddrInfoOptions.RELAY_AND_ADDRESSES)
    val doc1 = node1.docJoin(ticket)

    // Create author on node_1
    val author = node1.authorCreate()
    val blobSize = 100
    val bytes = generateRandomByteArray(blobSize)

    doc1.setBytes(author, "hello".toByteArray(Charsets.UTF_8), bytes)

    // Wait for the content ready event
    val hash = cb.channel.receive()
    println(hash)

    // Get content from hash
    val v = node1.blobsReadToBytes(hash)
    assert(bytes contentEquals v)
}
