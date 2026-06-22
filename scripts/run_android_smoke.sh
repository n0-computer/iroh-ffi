#!/usr/bin/env bash
# Run the IrohSmokeTest on a connected Android device/emulator.
#
# Used by both ci_kotlin.yml's kotlin-android-consumer-smoke job and
# release.yml's verify-kotlin-android-consumer job. Both invoke this from
# inside reactivecircus/android-emulator-runner's `script:` block, which
# runs each line as a separate `sh -c` — so multi-line scripts lose state
# across the cd. Putting the logic here keeps it in one shell session.

set -eu

V=$(sed -nE 's/.*coordinates\("computer\.iroh", "iroh", "([^"]+)"\).*/\1/p' kotlin/lib/build.gradle.kts)
echo "iroh version under test: $V"

cd kotlin/android-smoke
./gradlew :app:connectedDebugAndroidTest -PirohVersion="$V" --no-daemon --console=plain
