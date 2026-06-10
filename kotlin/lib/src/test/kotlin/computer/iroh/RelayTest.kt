package computer.iroh

import kotlin.test.Test

class RelayTest {
    @Test fun relayMapCrud() {
        val m = RelayMap.empty()
        assert(m.isEmpty())
        assert(m.len() == 0u)

        val cfg = RelayConfig(
            url = "https://relay.example.org/",
            quicPort = 7842u,
            authToken = "hunter2",
        )
        m.insert(cfg)
        assert(m.len() == 1u)
        assert(m.contains("https://relay.example.org/"))

        val got = m.get("https://relay.example.org/")
        assert(got != null)
        assert(got!!.url == "https://relay.example.org/")
        assert(got.quicPort == 7842u.toUShort())
        assert(got.authToken == "hunter2")

        assert(m.urls().contains("https://relay.example.org/"))
        assert(m.remove("https://relay.example.org/"))
        assert(m.isEmpty())
    }

    @Test fun relayMapFromUrls() {
        val m = RelayMap.fromUrls(listOf("https://r1.example.org/", "https://r2.example.org/"))
        assert(m.len() == 2u)
    }

    @Test fun relayModeConstructors() {
        RelayMode.disabled()
        RelayMode.defaultMode()
        RelayMode.staging()
        val m = RelayMap.fromUrls(listOf("https://r1.example.org/"))
        val custom = RelayMode.custom(m)
        assert(custom.relayMap().len() == 1u)
        RelayMode.customFromUrls(listOf("https://r2.example.org/"))
    }
}
