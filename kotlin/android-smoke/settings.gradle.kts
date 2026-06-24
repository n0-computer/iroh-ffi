// Standalone Android consumer build that exercises the JVM Maven artifact in
// its Android-merge path: AGP pulls lib/<abi>/*.so out of the iroh JAR
// dependency and packages it into the consumer APK, then
// System.loadLibrary("iroh_ffi") on the device picks it up.
//
// Uses a composite build to substitute `computer.iroh:iroh:<v>` with the
// in-tree `:lib` project from ../, so we don't need a publish step (or the
// signing key it requires under Vanniktech `signAllPublications()`). The lib's
// `:jar` task still runs, which is what AGP merge-jni-libs scans — so the
// bug class (JAR doesn't contain lib/<abi>/) is still caught.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

includeBuild("..") {
    dependencySubstitution {
        // Android consumers depend on computer.iroh:iroh-android (the AAR);
        // computer.iroh:iroh is the JVM-only JAR sibling, not what gets
        // pulled into APKs.
        substitute(module("computer.iroh:iroh-android")).using(project(":android"))
    }
}

rootProject.name = "iroh-android-smoke"
include("app")
