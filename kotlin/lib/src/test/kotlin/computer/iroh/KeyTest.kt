package computer.iroh

import kotlin.test.Test

class KeyTest {
    @Test fun basics() {
        val keyStr = "523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea"
        val fmtStr = "523c7996ba"
        val bytesU =
            ubyteArrayOf(
                0x52u,
                0x3cu,
                0x79u,
                0x96u,
                0xbau,
                0xd7u,
                0x74u,
                0x24u,
                0xe9u,
                0x67u,
                0x86u,
                0xcfu,
                0x7au,
                0x72u,
                0x05u,
                0x11u,
                0x53u,
                0x37u,
                0xa5u,
                0xb4u,
                0x56u,
                0x5cu,
                0xd2u,
                0x55u,
                0x06u,
                0xa0u,
                0xf2u,
                0x97u,
                0xb1u,
                0x91u,
                0xa5u,
                0xeau,
            )
        val bytes = bytesU.toByteArray()

        // create key from string
        val key = PublicKey.fromString(keyStr)

        // test methods are as expected
        assert(key.toString() == keyStr)
        assert(key.toBytes() contentEquals bytes)
        assert(key.fmtShort() == fmtStr)

        // create key from bytes
        val key0 = PublicKey.fromBytes(bytes)

        // test methods are as expected
        assert(key0.toString() == keyStr)
        assert(key0.toBytes() contentEquals bytes)
        assert(key0.fmtShort() == fmtStr)

        // test that the eq function works
        assert(key.equal(key0))
        assert(key0.equal(key))
    }
}
