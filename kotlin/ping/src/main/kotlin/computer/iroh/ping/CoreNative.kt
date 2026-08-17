package computer.iroh.ping

import com.sun.jna.NativeLibrary

/**
 * SPIKE: forces `libiroh_ffi` into the process before this package's own native library
 * is dlopen'd.
 *
 * The plugin library declares a dependency on `@rpath/libiroh_ffi.dylib` but ships no
 * rpath, because JNA extracts natives out of JARs to temp files with mangled names — so no
 * relative path can be relied on. dyld will satisfy the dependency from an *already loaded*
 * image matching that install name, so the only requirement is ordering.
 *
 * Without this, touching anything in this package first fails with `UnsatisfiedLinkError`.
 */
internal object CoreNative {
    init {
        // Order matters, and so does libstd. Every artifact here is a Rust `dylib` built
        // with -C prefer-dynamic, so all of them declare @rpath/libstd-<hash>.dylib. When
        // JNA extracts a native out of a JAR, nothing sits beside it in the temp dir, so
        // libstd has to be resolved by being loaded first too.
        NativeLibrary.getInstance("std-4f24f0876fd27385")
        NativeLibrary.getInstance("iroh_ffi")
    }

    /** Touch to force initialization. */
    fun ensureLoaded() {}
}

/** Loads the shared iroh native library. Call before using anything else in this package. */
fun initIrohPing() = CoreNative.ensureLoaded()
