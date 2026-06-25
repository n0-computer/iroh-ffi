package computer.iroh

import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertFailsWith

private val ALPN = "iroh-ffi/test/0".toByteArray()

/** A user-implemented Preset: minimal baseline + a custom ALPN. */
class CustomPreset : Preset {
    override fun apply(builder: EndpointBuilder) {
        builder.applyMinimal()
        builder.alpns(listOf("custom/preset/1".toByteArray()))
    }
}

class EndpointTest {
    @Test fun customPreset() = runBlocking {
        val ep = Endpoint.bind(EndpointOptions(preset = CustomPreset()))
        assert(ep.boundSockets().isNotEmpty())
        ep.shutdown()
    }

    @Test fun builderBind() = runBlocking {
        val builder = EndpointBuilder()
        builder.applyMinimal()
        val ep = builder.bind()
        assert(ep.boundSockets().isNotEmpty())
        ep.shutdown()
    }

    @Test fun builderBindConsumes() = runBlocking {
        val builder = EndpointBuilder()
        builder.applyMinimal()
        val ep = builder.bind()
        ep.shutdown()
        val err = assertFailsWith<Exception> {
            runBlocking { builder.bind() }
        }
        assert(err.toString().contains("already consumed")) { "got: $err" }
    }

    @Test fun bindLifecycle() = runBlocking {
        val ep = Endpoint.bind(EndpointOptions(preset = presetMinimal()))
        val id = ep.id()
        assert(id.toString().isNotEmpty())
        assert(ep.addr().id() == id)
        assert(ep.boundSockets().isNotEmpty())
        assert(ep.secretKey().public().toBytes() contentEquals id.toBytes())
        ep.shutdown()
        assert(ep.isClosed())
    }

    @Test fun endpointTicketRoundtrip() = runBlocking {
        val ep = Endpoint.bind(EndpointOptions(preset = presetMinimal()))
        val addr = ep.addr()
        val ticket = EndpointTicket.fromAddr(addr)
        val s = ticket.toString()
        assert(s.startsWith("endpoint"))
        val parsed = EndpointTicket.fromString(s)
        assert(parsed.endpointAddr().id() == addr.id())
        ep.shutdown()
    }

    @Test fun endpointTicketRejectsGarbage() {
        assertFailsWith<Exception> {
            EndpointTicket.fromString("not-a-ticket")
        }
    }

    @Test fun connectEchoRoundtrip() = runBlocking {
        val server = Endpoint.bind(
            EndpointOptions(
                preset = presetN0(),
                alpns = listOf(ALPN),
                relayMode = RelayMode.disabled(),
            ),
        )
        val serverAddr = server.addr()
        val serverId = server.id()

        val serverJob = async {
            val incoming = server.acceptNext()!!
            val conn = incoming.accept().connect()
            assert(conn.alpn() contentEquals ALPN)
            val bi = conn.acceptBi()
            val msg = bi.recv().readToEnd(64u)
            bi.send().writeAll(msg)
            bi.send().finish()
            val dg = conn.readDatagram()
            conn.sendDatagram(dg)
            conn.closed()
        }

        val client = Endpoint.bind(
            EndpointOptions(preset = presetN0(), relayMode = RelayMode.disabled()),
        )
        val conn = client.connect(serverAddr, ALPN)
        assert(conn.remoteId() == serverId)
        assert(conn.paths().isNotEmpty())

        val bi = conn.openBi()
        bi.send().writeAll("hello iroh".toByteArray())
        bi.send().finish()
        val echoed = bi.recv().readToEnd(64u)
        assert(String(echoed) == "hello iroh")

        conn.sendDatagram("ping".toByteArray())
        val pong = conn.readDatagram()
        assert(String(pong) == "ping")

        val stats = conn.stats()
        assert(stats.udpTxDatagrams > 0)

        conn.close(0, "bye".toByteArray())
        serverJob.await()
        client.shutdown()
        server.shutdown()
    }

    @Test fun uniStream() = runBlocking {
        val server = Endpoint.bind(
            EndpointOptions(
                preset = presetN0(),
                alpns = listOf(ALPN),
                relayMode = RelayMode.disabled(),
            ),
        )
        val serverAddr = server.addr()

        val serverJob = async {
            val incoming = server.acceptNext()!!
            val conn = incoming.accept().connect()
            val recv = conn.acceptUni()
            val msg = recv.readToEnd(32u)
            assert(String(msg) == "unidirectional")
        }

        val client = Endpoint.bind(
            EndpointOptions(preset = presetN0(), relayMode = RelayMode.disabled()),
        )
        val conn = client.connect(serverAddr, ALPN)
        val send = conn.openUni()
        send.writeAll("unidirectional".toByteArray())
        send.finish()

        serverJob.await()
        client.shutdown()
        server.shutdown()
    }
}
