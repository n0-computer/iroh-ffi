package computer.iroh

import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertFailsWith

// Well-formed (but fake) API secret — the remote does not exist, but the
// client connects lazily so construction still succeeds.
private const val FAKE_API_SECRET =
    "servicesaaqaobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqob" +
        "75c4sdqwvay5nwj63yzvqc7iozsh66x53lcpcy5vyc5ledl2pwdaaa"

class ServicesTest {
    private suspend fun endpoint() =
        Endpoint.bind(EndpointOptions(preset = presetMinimal()))

    @Test fun bootsWithFakeSecret() = runBlocking {
        val ep = endpoint()
        ServicesClient.create(ep, ServicesOptions(apiSecret = FAKE_API_SECRET))
        ep.shutdown()
    }

    @Test fun rejectsNoCredentials() = runBlocking {
        val ep = endpoint()
        assertFailsWith<Exception> {
            ServicesClient.create(ep, ServicesOptions())
        }
        ep.shutdown()
    }

    @Test fun rejectsTwoCredentials() = runBlocking {
        val ep = endpoint()
        assertFailsWith<Exception> {
            ServicesClient.create(
                ep,
                ServicesOptions(apiSecret = FAKE_API_SECRET, apiSecretFromEnv = true),
            )
        }
        ep.shutdown()
    }

    @Test fun rejectsMalformedSecret() = runBlocking {
        val ep = endpoint()
        assertFailsWith<Exception> {
            ServicesClient.create(ep, ServicesOptions(apiSecret = "not-a-valid-ticket"))
        }
        ep.shutdown()
    }
}
