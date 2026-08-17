package computer.iroh.ping

import computer.iroh.Endpoint
import computer.iroh.EndpointOptions
import computer.iroh.ProtocolCreator
import computer.iroh.ProtocolHandler
import computer.iroh.presetMinimal
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * SPIKE: proves an Endpoint minted by `computer.iroh`'s native library is usable by
 * `computer.iroh.ping`'s separately-built native library, with one copy of iroh in the
 * process. See spikes/FINDINGS.md.
 */
class PingCreator(private val ping: Ping) : ProtocolCreator {
    override fun create(endpoint: Endpoint): ProtocolHandler = ping.handler()
}

class PingTest {
    @Test
    fun crossLibraryPing() = runBlocking {
        withTimeout(60_000) {
            val ping = Ping()

            // Server: endpoint from the CORE library, handler from the PLUGIN library.
            // `handler()` returns a computer.iroh.ProtocolHandler — a callback-interface
            // handle crossing between the two native libraries.
            val server = Endpoint.bind(
                EndpointOptions(
                    preset = presetMinimal(),
                    protocols = mapOf(alpn() to PingCreator(ping)),
                ),
            )
            val serverAddr = server.addr()
            println("  server bound: $serverAddr")

            val client = Endpoint.bind(EndpointOptions(preset = presetMinimal()))
            println("  client bound: ${client.addr()}")

            val rttMs = ping.ping(client, serverAddr)
            println("  PING -> PONG round trip: $rttMs ms")
            assertTrue(rttMs < 60_000uL, "implausible rtt: $rttMs")

            client.shutdown()
            server.shutdown()
            println("  OK: Endpoint crossed the library boundary and the protocol ran")
        }
    }

    /**
     * The ALPN comes from the plugin's own namespace, not core's — so this test touches the
     * PLUGIN's native library first and never touches core's. Without [initIrohPing] it
     * fails with `UnsatisfiedLinkError`, because the plugin ships no rpath to core and dyld
     * can only satisfy `@rpath/libiroh_ffi.dylib` from an already-loaded image.
     */
    @Test
    fun alpnFromPluginLoadedFirst() {
        initIrohPing()
        assertTrue(alpn().isNotEmpty())
        println("  alpn = ${String(alpn())}")
    }
}
