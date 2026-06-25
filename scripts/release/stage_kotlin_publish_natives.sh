#!/usr/bin/env bash
# Stage the per-platform native libs into both Kotlin module source trees
# ahead of the Maven Central publish:
#
#   - :lib (JVM JAR) — desktop cdylibs under JNA-platform subdirs of
#     kotlin/lib/src/main/resources/. JNA discovers libs from classpath at
#     <jna-platform>/<libname>.{so,dylib,dll}.
#   - :android (AAR) — Android .so files under
#     kotlin/android/src/main/jniLibs/<abi>/. AGP packages these into the
#     AAR's jni/<abi>/ which consumers' APKs then merge.
#
# Expects upstream CI to have downloaded:
#   - cdylibs/kotlin-cdylib-{linux-x86_64,linux-aarch64,macos-aarch64,windows-x86_64}/
#   - android-jniLibs/{armeabi-v7a,arm64-v8a,x86,x86_64}/

set -eu

RES=kotlin/lib/src/main/resources
rm -f "$RES"/libiroh_ffi.so "$RES"/libiroh_ffi.dylib "$RES"/iroh_ffi.dll
install -D -m644 cdylibs/kotlin-cdylib-linux-x86_64/libiroh_ffi.so     "$RES/linux-x86-64/libiroh_ffi.so"
install -D -m644 cdylibs/kotlin-cdylib-linux-aarch64/libiroh_ffi.so    "$RES/linux-aarch64/libiroh_ffi.so"
install -D -m644 cdylibs/kotlin-cdylib-macos-aarch64/libiroh_ffi.dylib "$RES/darwin-aarch64/libiroh_ffi.dylib"
install -D -m644 cdylibs/kotlin-cdylib-windows-x86_64/iroh_ffi.dll     "$RES/win32-x86-64/iroh_ffi.dll"

JNI=kotlin/android/src/main/jniLibs
for abi in armeabi-v7a arm64-v8a x86 x86_64; do
  install -D -m644 "android-jniLibs/$abi/libiroh_ffi.so" "$JNI/$abi/libiroh_ffi.so"
done

find "$RES" "$JNI" -type f
