#!/usr/bin/env bash
# Run the Android consumer smoke. Used by ci_kotlin.yml and release.yml
# inside reactivecircus/android-emulator-runner's `script:` block (which
# runs each line as a separate `sh -c`, so multi-line logic lives here).

set -eu
cd kotlin
./gradlew :android-smoke:connectedDebugAndroidTest --no-daemon --console=plain
