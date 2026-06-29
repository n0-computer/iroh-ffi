#!/usr/bin/env bash
# Stage Android native libs from $JNI_SOURCE (default ./android-jniLibs) into
# kotlin/android/src/main/jniLibs/<abi>/ — the AAR source-set path AGP packages
# into the AAR's jni/<abi>/. Also runs bindgen-kotlin so the composite-build
# :android:assemble has iroh_ffi.kt for :lib.
#
# ci_kotlin.yml's smoke job cross-builds with cargo ndk into ./android-jniLibs/
# then calls this. release.yml's job downloads the build-kotlin-android
# artifact to the same path then calls this. Both paths land at the same AAR
# source-set tree.

set -eu

JNI_SOURCE="${JNI_SOURCE:-./android-jniLibs}"
[ -d "$JNI_SOURCE" ] || { echo "ERROR: $JNI_SOURCE not found — cross-build or download the artifact first" >&2; exit 1; }

JNI_DEST=kotlin/android/src/main/jniLibs
for abi in armeabi-v7a arm64-v8a x86 x86_64; do
  install -D -m644 "$JNI_SOURCE/$abi/libiroh_ffi.so" "$JNI_DEST/$abi/libiroh_ffi.so"
done

cargo make bindgen-kotlin
