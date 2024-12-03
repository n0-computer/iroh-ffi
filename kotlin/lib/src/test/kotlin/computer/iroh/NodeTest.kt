package computer.iroh

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test

class Subscriber : SubscribeCallback {
    val channel = Channel<LiveEvent>(8)

    override suspend fun `event`(`event`: LiveEvent) {
        println(event.type())
        this.channel.send(event)
    }
}

class MyProtocol : ProtocolHandler {
    override suspend fun accept(connecting: Connecting) {
        val conn = connecting.connect()
        val remote = conn.getRemoteNodeId()
        println("accepting from $remote")
        val bi = conn.acceptBi()

        val bytes = bi.recv().readToEnd(64u)
        val b = bytes.toString(Charsets.UTF_8)
        println("got $b")
        assert("yo".toByteArray(Charsets.UTF_8) contentEquals bytes)
        bi.send().writeAll("hello".toByteArray(Charsets.UTF_8))
        bi.send().finish()
        bi.send().stopped()
    }

    override suspend fun shutdown() {
        println("shutting down")
    }
}

class MyProtocolCreator : ProtocolCreator {
    override fun create(endpoint: Endpoint): MyProtocol = MyProtocol()

    companion object
}

class NodeTest {
    @Test fun basics() =
        runTest {
            // setLogLevel(LogLevel.DEBUG)
            // Create node_0
            val options = NodeOptions()
            options.enableDocs = true

            val node0 = Iroh.memoryWithOptions(options)

            // Create node_1
            val node1 = Iroh.memoryWithOptions(options)

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
            while (true) {
                val event = cb1.channel.receive()
                if (event.type() == LiveEventType.SYNC_FINISHED) {
                    break
                }
            }

            // Create author on node_1
            val author = node1.authors().create()
            val blobSize = 100
            val bytes = generateRandomByteArray(blobSize)

            doc1.setBytes(author, "hello".toByteArray(Charsets.UTF_8), bytes)

            // Wait for the content ready event
            while (true) {
                val event = cb0.channel.receive()
                if (event.type() == LiveEventType.CONTENT_READY) {
                    val hash = event.asContentReady()
                    println(hash)

                    // Get content from hash
                    val v = node1.blobs().readToBytes(hash)
                    assert(bytes contentEquals v)

                    break
                }
            }

            node1.node().shutdown()
            node0.node().shutdown()
        }

    @Test fun simpleProtocol() =
        runTest {
            val protocols =
                hashMapOf(
                    "example/protocol/0".toByteArray(Charsets.UTF_8)
                        to
                        MyProtocolCreator(),
                )

            val options = NodeOptions()
            options.protocols = protocols

            // Create node1
            val node1 = Iroh.memoryWithOptions(options)

            // Create node2
            val node2 = Iroh.memoryWithOptions(options)

            val alpn = "example/protocol/0".toByteArray(Charsets.UTF_8)
            val nodeAddr = node1.net().nodeAddr()

            val endpoint = node2.node().endpoint()
            val conn = endpoint.connect(nodeAddr, alpn)
            val remote = conn.getRemoteNodeId()
            println(remote)

            val bi = conn.openBi()

            bi.send().writeAll("yo".toByteArray(Charsets.UTF_8))
            bi.send().finish()
            bi.send().stopped()

            val o = bi.recv().readExact(5u)
            println(o.toString(Charsets.UTF_8))
            assert("hello".toByteArray(Charsets.UTF_8) contentEquals o)

            node2.node().shutdown()
            node1.node().shutdown()
        }
}
