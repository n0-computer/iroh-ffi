// Root build is intentionally empty — the AGP + Kotlin plugins are applied
// in :app. Kotlin version must match kotlin/lib (2.2.20 per
// kotlin/gradle/libs.versions.toml); the lib's compiled metadata is 2.2.0
// and a Kotlin 2.0 consumer rejects it. AGP 8.13 supports Kotlin 2.2.x and
// requires Gradle 8.13+ (matching gradle-wrapper.properties below).
plugins {
    id("com.android.application") version "8.13.0" apply false
    id("org.jetbrains.kotlin.android") version "2.2.20" apply false
}
