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
    val channel = Channel<LiveEvent>(8)

    override suspend fun `event`(`event`: LiveEvent) {
        this.channel.send(event)
    }
}

runBlocking {
    // setLogLevel(LogLevel.DEBUG)
    // Create node_0
    val node0 = Iroh.memory()

    // Create node_1
    val node1 = Iroh.memory()

    println("setup node0")

    // Create doc on node_0
    val doc0 = node0.docs().create()

    // Subscribe to sync events
    val cb0 = Subscriber()
    doc0.subscribe(cb0)

    // Join the same doc from node_1
    val ticket = doc0.share(ShareMode.WRITE, AddrInfoOptions.RELAY_AND_ADDRESSES)
    val cb1 = Subscriber()
    val doc1 = node1.docs().joinAndSubscribe(ticket, cb1)

    // wait for initial sync
    println("waiting for sync")
    while (true) {
        val event = cb1.channel.receive()
        println("node1: " + event.type())
        if (event.type() == LiveEventType.SYNC_FINISHED) {
            break
        }
    }

    println("setup node1")

    // Create author on node_1
    val author = node1.authors().create()
    val blobSize = 100
    val bytes = generateRandomByteArray(blobSize)

    doc1.setBytes(author, "hello".toByteArray(Charsets.UTF_8), bytes)

    // Wait for the content ready event
    while (true) {
        val event = cb0.channel.receive()
        println("node0: " + event.type())
        if (event.type() == LiveEventType.CONTENT_READY) {
            val hash = event.asContentReady()
            println("received hash: " + hash)

            // Get content from hash
            val v = node1.blobs().readToBytes(hash)
            assert(bytes contentEquals v)

            break
        }
    }
}
