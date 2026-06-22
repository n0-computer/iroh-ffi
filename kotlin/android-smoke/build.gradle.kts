// Root build is intentionally empty — the AGP + Kotlin plugins are applied
// in :app. Pin versions in one place so consumer cards (settings + app) line
// up; AGP 8.7 needs Gradle 8.11+ and Kotlin 2.0+.
plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
}
