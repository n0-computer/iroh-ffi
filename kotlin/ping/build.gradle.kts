// SPIKE: stands in for an independently published `computer.iroh:iroh-ping`.
//
// It depends on `:lib` (computer.iroh) for the shared types — Endpoint, EndpointAddr,
// ProtocolHandler, and crucially RustBuffer — and ships only its own small native library.
// See spikes/FINDINGS.md.

plugins {
    alias(libs.plugins.kotlin.jvm)
    `java-library`
}

repositories {
    mavenCentral()
}

dependencies {
    // `api`, not `implementation`: the generated bindings expose computer.iroh types in
    // their public signatures (e.g. `suspend fun ping(endpoint: Endpoint, ...)`).
    api(project(":lib"))

    implementation("net.java.dev.jna:jna:5.15.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.9.0")

    testImplementation("org.jetbrains.kotlin:kotlin-test-junit5")
    testImplementation(libs.junit.jupiter.engine)
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.9.0")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}

kotlin {
    compilerOptions {
        optIn.add("kotlin.ExperimentalUnsignedTypes")
    }
}

tasks.named<Test>("test") {
    useJUnitPlatform()

    // Absolute: a Test task's workingDir is the *project* dir (kotlin/ping), so relative
    // paths here silently miss and JNA falls back to extracting from the classpath.
    val nativePath = listOf(
        rootDir.resolve("lib/src/main/resources"),
        rootDir.resolve("ping/src/main/resources"),
    ).joinToString(File.pathSeparator) { it.absolutePath }
    systemProperty("java.library.path", nativePath)
    systemProperty("jna.library.path", nativePath)

    testLogging {
        events("passed", "skipped", "failed")
        showStandardStreams = true
    }
}

// SPIKE: the *published* path. With no jna.library.path, JNA extracts each native out of
// its JAR into a temp file with a mangled name, which is what a real consumer gets.
// Characterizes whether the two-artifact split survives extraction.
tasks.register<Test>("testExtracted") {
    testClassesDirs = sourceSets["test"].output.classesDirs
    classpath = sourceSets["test"].runtimeClasspath
    useJUnitPlatform()
    systemProperty("jna.library.path", layout.buildDirectory.dir("empty-native").get().asFile.absolutePath)
    testLogging {
        events("passed", "skipped", "failed")
        showStandardStreams = true
        showExceptions = true
        showCauses = true
    }
}
