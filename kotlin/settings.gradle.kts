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
include("lib", "android")
