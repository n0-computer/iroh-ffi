// Declare every plugin at the root with `apply false` so that all subprojects
// share one classloader scope per plugin. Without this, Vanniktech's
// MavenCentralBuildService gets loaded into two separate classloaders (one
// for :lib, one for :android) and Gradle errors with "Cannot set the value
// of task 'prepareMavenCentralPublishing' property 'buildService'".
plugins {
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.dokka) apply false
    alias(libs.plugins.maven.publish) apply false
}
