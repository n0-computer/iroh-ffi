package computer.iroh.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import computer.iroh.IrohAndroid
import computer.iroh.SecretKey
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.junit.runner.RunWith

/**
 * On-device smoke for `computer.iroh:iroh:<version>`. If `lib/<abi>/libiroh_ffi.so`
 * isn't merged into this consumer APK by AGP (issue #246), or it loads but a
 * uniffi symbol is missing, this test fails before the assertion.
 *
 * Stays entirely offline — `SecretKey.generate()` is local ed25519, no network.
 */
@RunWith(AndroidJUnit4::class)
class IrohSmokeTest {

    @Test
    fun installAndroidContext_loadsNativeLib() {
        // The `init` block on `IrohAndroid` calls `System.loadLibrary("iroh_ffi")`.
        // Touching the object triggers init → throws UnsatisfiedLinkError if the
        // .so isn't packaged at lib/<abi>/ or doesn't load on this device.
        val ctx = InstrumentationRegistry.getInstrumentation().targetContext
        IrohAndroid.installAndroidContext(ctx)
    }

    @Test
    fun secretKey_roundtripsThroughJni() {
        // Exercises the uniffi-generated bridge end-to-end: object construction,
        // a method call returning a wrapped object, and a method returning bytes.
        // `public` is a Kotlin keyword — uniffi emits it backtick-escaped.
        val secret = SecretKey.generate()
        val pub = secret.`public`()
        assertNotNull(pub)
        assertEquals(32, pub.toBytes().size)
    }
}
