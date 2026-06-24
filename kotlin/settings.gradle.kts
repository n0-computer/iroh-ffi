// Multi-project: `:lib` ships `computer.iroh:iroh` (the JVM-only JAR for
// Java/Kotlin desktop apps), `:android` ships `computer.iroh:iroh-android`
// (the AAR with Android natives + IrohAndroid for Android consumers). The
// AAR transitively depends on the JAR — Android consumers get all the iroh
// API plus libiroh_ffi.so under jni/<abi>/.

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

// `:android` requires an Android SDK at Gradle configuration time. On hosts
// without one (e.g. the self-hosted linux-aarch64 runner where
// android-actions/setup-android fails), skip the subproject so the rest of
// the build still works. Jobs that need :android (publish, Android consumer
// smoke) run setup-android before invoking Gradle.
val androidSdkAvailable = listOf(
    System.getenv("ANDROID_HOME"),
    System.getenv("ANDROID_SDK_ROOT"),
).any { !it.isNullOrEmpty() } || file("local.properties").exists()
if (androidSdkAvailable) {
    include("android")
} else {
    logger.lifecycle("[settings] No Android SDK detected (ANDROID_HOME / ANDROID_SDK_ROOT unset, no local.properties) — skipping :android subproject")
}
