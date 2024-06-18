// tests that correspond to the `src/doc.rs` rust api

import iroh.*
import java.util.concurrent.ArrayBlockingQueue

// Create node_0
val irohDir0 = kotlin.io.path.createTempDirectory("node-test-0")
val node0 = IrohNode(irohDir0.toString())

// Create node_1
val irohDir1 = kotlin.io.path.createTempDirectory("node-test-1")
val node1 = IrohNode(irohDir1.toString())

// Create doc on node_0
val doc0 = node0.docCreate()
val ticket = doc0.share(ShareMode.WRITE, AddrInfoOptions.RELAY_AND_ADDRESSES)

class Subscriber : SubscribeCallback {
    val channel = ArrayBlockingQueue<Hash>(1)

    override fun `event`(`event`: LiveEvent) {
        println(event.type())
        if (event.type() == LiveEventType.CONTENT_READY) {
            println("got event type content ready")
            this.channel.put(event.asContentReady())
        }
    }
}

// Subscribe to sync events
val cb = Subscriber()
doc0.subscribe(cb)

// Join the same doc from node_1
val doc1 = node1.docJoin(ticket)

// Create author on node_1
val author = node1.authorCreate()
doc1.setBytes(author, "hello".toByteArray(Charsets.UTF_8), "world".toByteArray(Charsets.UTF_8))

// Wait for the content ready event
val hash = cb.channel.take()

// Get content from hash
val v = node1.blobsReadToBytes(hash)
assert("world" contentEquals v.toString(Charsets.UTF_8))
