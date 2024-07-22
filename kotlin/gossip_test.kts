// tests that correspond to the `src/gossp.rs` rust api

import iroh.*
import kotlinx.coroutines.channels.*
import kotlinx.coroutines.runBlocking

class Callback: GossipMessageCallback {
    val channel = Channel<Message>(8)

    override suspend fun onMessage(msg: Message) {
        println(msg.type())
        this.channel.send(msg)
    }
}

runBlocking {
    setLogLevel(LogLevel.DEBUG)

    val n0 = Iroh.memory()
    val n1 = Iroh.memory()

    // Create a topic
    val topic = ByteArray(32) { i -> 1 }

    // Setup gossip on node 0
    val cb0 = Callback()
    val n1Id = n1.node().nodeId()
    val n1Addr = n1.node().nodeAddr()
    n0.node().addNodeAddr(n1Addr)

    println("subscribe n0")
    val sink0 = n0.gossip().subscribe(topic, listOf(n1Id), cb0)

    // Setup gossip on node 1
    val cb1 = Callback()
    val n0Id = n0.node().nodeId()
    val n0Addr = n0.node().nodeAddr()
    n1.node().addNodeAddr(n0Addr)

    println("subscribe n1")
    val sink1 = n1.gossip().subscribe(topic, listOf(n0Id), cb1)

    // Wait for n1 to show up for n0
    while (true) {
        val event = cb0.channel.receive()
        println(event.type())
        if (event.type() == MessageType.NEIGHBOR_UP) {
            assert(event.asNeighborUp() contentEquals n1Id)
            break
        }
    }

    // Broadcact message from node 0
    println("broadcasting message")
    val msg_content = "hello".toByteArray(Charsets.UTF_8)

    sink0.broadcast(msg_content)

    // Wait for message on n1
    var found = false

    // Wait for the message on node 1
    while (true) {
        val event = cb1.channel.receive()
        if (event.type() == MessageType.RECEIVED) {
            val msg = event.asReceived()
            assert(msg.content contentEquals msg_content)
            assert(msg.deliveredFrom contentEquals n0Id)
            found = true
            break
        }
    }

    assert(found)
}
