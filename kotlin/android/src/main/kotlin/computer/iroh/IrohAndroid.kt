package computer.iroh

import android.content.Context

/**
 * Android-specific iroh initialization. Apps must call
 * [installAndroidContext] once during startup (typically from
 * `Application.onCreate` or before constructing any [Endpoint]) so
 * that iroh's DNS resolver can reach Android's [`LinkProperties`] via
 * JNI.
 *
 * The call is idempotent — subsequent invocations are no-ops.
 */
object IrohAndroid {
    init {
        // libiroh_ffi exposes the JNI entry point Java_computer_iroh_IrohAndroid_installAndroidContext.
        // Loading the library here makes the symbol available before the first external call.
        System.loadLibrary("iroh_ffi")
    }

    @JvmStatic
    external fun installAndroidContext(context: Context)
}
