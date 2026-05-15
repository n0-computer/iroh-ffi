package computer.iroh

import kotlin.test.Test
import kotlin.test.assertFailsWith

class KeyTest {
    @Test fun endpointId() {
        val keyStr = "523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea"
        val fmtStr = "523c7996ba"
        val bytesU =
            ubyteArrayOf(
                0x52u, 0x3cu, 0x79u, 0x96u, 0xbau, 0xd7u, 0x74u, 0x24u,
                0xe9u, 0x67u, 0x86u, 0xcfu, 0x7au, 0x72u, 0x05u, 0x11u,
                0x53u, 0x37u, 0xa5u, 0xb4u, 0x56u, 0x5cu, 0xd2u, 0x55u,
                0x06u, 0xa0u, 0xf2u, 0x97u, 0xb1u, 0x91u, 0xa5u, 0xeau,
            )
        val bytes = bytesU.toByteArray()

        val id = EndpointId.fromString(keyStr)
        assert(id.toString() == keyStr)
        assert(id.toBytes() contentEquals bytes)
        assert(id.fmtShort() == fmtStr)

        val id2 = EndpointId.fromBytes(bytes)
        assert(id2.toString() == keyStr)
        assert(id2.toBytes() contentEquals bytes)
        assert(id2.fmtShort() == fmtStr)

        assert(id.equal(id2))
        assert(id2.equal(id))
    }

    @Test fun endpointIdRejectsBadBytes() {
        assertFailsWith<Exception> {
            EndpointId.fromBytes(byteArrayOf(1, 2, 3))
        }
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
