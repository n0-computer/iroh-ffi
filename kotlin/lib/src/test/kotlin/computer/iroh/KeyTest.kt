package computer.iroh

import kotlin.test.Test
import kotlin.test.assertFailsWith

class KeyTest {
    @Test fun endpointId() {
        val keyStr = "523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea"
        val fmtStr = "523c7996ba"
        val bytes = keyStr.hexToByteArray()

        val id = EndpointId.fromString(keyStr)
        assert(id.toString() == keyStr)
        assert(id.toBytes() contentEquals bytes)
        assert(id.fmtShort() == fmtStr)

        val id2 = EndpointId.fromBytes(bytes)
        assert(id2.toString() == keyStr)
        assert(id2.toBytes() contentEquals bytes)
        assert(id2.fmtShort() == fmtStr)

        assert(id == id2)
        assert(id2 == id)
    }

    @Test fun endpointIdRejectsBadBytes() {
        val err = assertFailsWith<IrohException> {
            EndpointId.fromBytes(byteArrayOf(1, 2, 3))
        }
        assert(err.kind() == IrohErrorKind.INVALID_INPUT)
        assert(err.isKind(IrohErrorKind.INVALID_INPUT))
        assert(err.message().contains("32 bytes"))
        assert(err.debugMessage() == err.message())
    }

    @Test fun endpointIdParseErrorKind() {
        val err = assertFailsWith<IrohException> {
            EndpointId.fromString("not-an-endpoint-id")
        }
        assert(err.kind() == IrohErrorKind.KEY_PARSING)
        assert(err.isKind(IrohErrorKind.KEY_PARSING))
        assert(err.message().isNotEmpty())
        assert(err.debugMessage().isNotEmpty())
    }

    @Test fun secretKeyRoundtrip() {
        val secret = SecretKey.generate()
        val raw = secret.toBytes()
        assert(raw.size == 32)
        val secret2 = SecretKey.fromBytes(raw)
        assert(secret.toBytes() contentEquals secret2.toBytes())
        assert(secret.public().toBytes() contentEquals secret2.public().toBytes())
    }

    @Test fun signVerifyRoundtrip() {
        val secret = SecretKey.generate()
        val pub = secret.public()
        val msg = "hello iroh".toByteArray()
        val sig = secret.sign(msg)

        val raw = sig.toBytes()
        assert(raw.size == 64)
        val sig2 = Signature.fromBytes(raw)
        assert(sig2.toBytes() contentEquals raw)

        pub.verify(msg, sig)
        pub.verify(msg, sig2)
    }

    @Test fun verifyRejectsTampered() {
        val secret = SecretKey.generate()
        val pub = secret.public()
        val sig = secret.sign("original".toByteArray())
        assertFailsWith<Exception> {
            pub.verify("tampered".toByteArray(), sig)
        }
    }
}
