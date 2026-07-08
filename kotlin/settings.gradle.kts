// `:lib` ships computer.iroh:iroh (JVM JAR); `:android` ships
// computer.iroh:iroh-android (AAR with libiroh_ffi.so + IrohAndroid).
// `:android-smoke` is a test consumer of the AAR, not published.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.8.0"
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "iroh-kotlin"
include("lib")

// `:android` + `:android-smoke` require the Android SDK at configuration time.
// On hosts without one (the self-hosted linux-aarch64 runner where setup-android
// fails), skip both so `:lib` still builds + tests.
val androidSdkAvailable = listOf(
    System.getenv("ANDROID_HOME"),
    System.getenv("ANDROID_SDK_ROOT"),
).any { !it.isNullOrEmpty() } || file("local.properties").exists()
if (androidSdkAvailable) {
    include("android", "android-smoke")
} else {
    logger.lifecycle("[settings] No Android SDK detected — skipping :android and :android-smoke")
}
