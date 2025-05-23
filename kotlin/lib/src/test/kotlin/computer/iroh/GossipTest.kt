package computer.iroh

import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test

class Callback : GossipMessageCallback {
    val channel = Channel<Message>(8)

    override suspend fun onMessage(msg: Message) {
        println(msg.type())
        this.channel.send(msg)
    }
}

class GossipTest {
    @Test fun basics() =
        runTest {
            // setLogLevel(LogLevel.DEBUG)

            val n0 = Iroh.memory()
            val n1 = Iroh.memory()

            // Create a topic
            val topic = ByteArray(32) { i -> 1 }

            // Setup gossip on node 0
            val cb0 = Callback()
            val n1Id = n1.net().nodeId()
            val n1Addr = n1.net().nodeAddr()
            n0.net().addNodeAddr(n1Addr)

            println("subscribe n0")
            val sink0 = n0.gossip().subscribe(topic, listOf(n1Id), cb0)

            // Setup gossip on node 1
            val cb1 = Callback()
            val n0Id = n0.net().nodeId()
            val n0Addr = n0.net().nodeAddr()
            n1.net().addNodeAddr(n0Addr)

            println("subscribe n1")
            val sink1 = n1.gossip().subscribe(topic, listOf(n0Id), cb1)

            // Wait for n1 to show up for n0
            while (true) {
                val event = cb0.channel.receive()
                println(event.type())
                if (event.type() == MessageType.JOINED) {
                    break
                }
            }

            // Broadcact message from node 0
            println("broadcasting message")
            val msgContent = "hello".toByteArray(Charsets.UTF_8)

            sink0.broadcast(msgContent)

            // Wait for message on n1
            var found = false

            // Wait for the message on node 1
            while (true) {
                val event = cb1.channel.receive()
                if (event.type() == MessageType.RECEIVED) {
                    val msg = event.asReceived()
                    assert(msg.content contentEquals msgContent)
                    assert(msg.deliveredFrom contentEquals n0Id)
                    found = true
                    break
                }
            }

            assert(found)

            sink0.cancel()
            sink1.cancel()

            n0.node().shutdown()
            n1.node().shutdown()
        }
}
