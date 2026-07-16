// iroh-android: the AAR companion to the JVM-only iroh JAR. Android consumers
// depend on this artifact instead of (or in addition to) computer.iroh:iroh —
// the AAR transitively brings in the JAR's Kotlin API plus the per-ABI
// libiroh_ffi.so files at the path AGP packages into consumer APKs.

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.dokka)
    alias(libs.plugins.maven.publish)
}

android {
    namespace = "computer.iroh.android"
    compileSdk = 34

    defaultConfig {
        // 24 matches what consumer apps typically use; cargo-ndk's default
        // API level (21) is below this so the .so files load on minSdk hosts.
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    // Brings in the full computer.iroh:iroh Kotlin API (Endpoint, SecretKey,
    // …). Excluding JNA from this transitive: the JVM JAR depends on
    // net.java.dev.jna:jna:5.15.0 (the JAR variant, fine for desktop); the
    // AAR variant — which carries libjnidispatch.so per ABI — is added
    // explicitly below. Without the exclude, AGP errors on duplicate
    // com.sun.jna.* classes from the JAR + AAR.
    api(project(":lib")) {
        exclude(group = "net.java.dev.jna", module = "jna")
    }
    api("net.java.dev.jna:jna:5.15.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.9.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
}

// Vanniktech 0.34: AndroidSingleVariantLibrary publishes the `release`
// build variant + a sources JAR + a Dokka-generated javadoc JAR. Same
// shape as the JVM publication in :lib, just an AAR instead of a JAR.
mavenPublishing {
    publishToMavenCentral(automaticRelease = true)
    signAllPublications()
    configure(
        com.vanniktech.maven.publish.AndroidSingleVariantLibrary(
            variant = "release",
            sourcesJar = true,
            publishJavadocJar = true,
        ),
    )
    coordinates("computer.iroh", "iroh-android", "1.1.0")
    pom {
        name = "iroh-android"
        description = "Android bindings for iroh: distributed systems made simple. AAR variant of computer.iroh:iroh — includes libiroh_ffi.so per ABI + IrohAndroid for JNI initialization."
        url = "https://github.com/n0-computer/iroh-ffi"
        licenses {
            license {
                name = "MIT"
                url = "https://opensource.org/license/mit"
            }
            license {
                name = "Apache-2.0"
                url = "https://www.apache.org/licenses/LICENSE-2.0"
            }
        }
        developers {
            developer {
                id = "n0-computer"
                name = "n0"
                url = "https://www.iroh.computer"
            }
        }
        scm {
            url = "https://github.com/n0-computer/iroh-ffi"
            connection = "scm:git:git://github.com/n0-computer/iroh-ffi.git"
            developerConnection = "scm:git:ssh://git@github.com/n0-computer/iroh-ffi.git"
        }
    }
}
