plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

// Override at invoke time: `./gradlew -PirohVersion=1.0.0`. The verify task
// reads the current version straight from kotlin/lib/build.gradle.kts and
// passes it here so a developer can't accidentally test against a stale
// version cached in mavenLocal.
val irohVersion: String =
    (project.findProperty("irohVersion") as String?) ?: "1.0.0"

android {
    namespace = "computer.iroh.smoke"
    compileSdk = 34

    defaultConfig {
        applicationId = "computer.iroh.smoke"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    // The artifact under test.
    implementation("computer.iroh:iroh:$irohVersion")
    // JNA isn't transitively resolved through some Android-side metadata
    // quirks; force it so the JNI binding finds it at install time.
    implementation("net.java.dev.jna:jna:5.15.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")

    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test:runner:1.6.2")
    androidTestImplementation("androidx.test:rules:1.6.1")
}
