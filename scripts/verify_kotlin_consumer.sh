#!/usr/bin/env bash
# JVM Maven artifact consumer smoke. Builds the host's release cdylib, stages
# it under its JNA-platform subdir (the path JNA discovers from classpath at
# runtime, same as release.yml's stage step), hides the dev-mode flat .so so
# the per-platform path must resolve, then runs the gradle test suite.
#
# Cross-platform — runs on linux/macOS/windows (uses Git Bash on win32).
# Lives outside Makefile.toml so cargo-make doesn't try to run the script
# through cmd.exe on Windows.

set -eu

RES=kotlin/lib/src/main/resources

case "$(uname -sm)" in
  "Darwin arm64")     PLAT=darwin-aarch64 ; LIB=libiroh_ffi.dylib ;;
  "Darwin x86_64")    PLAT=darwin-x86-64  ; LIB=libiroh_ffi.dylib ;;
  "Linux x86_64")     PLAT=linux-x86-64   ; LIB=libiroh_ffi.so    ;;
  "Linux aarch64")    PLAT=linux-aarch64  ; LIB=libiroh_ffi.so    ;;
  MINGW*\ x86_64|MSYS*\ x86_64)
                      PLAT=win32-x86-64   ; LIB=iroh_ffi.dll      ;;
  *) echo "ERROR: unsupported host $(uname -sm) — add a case branch" >&2; exit 1 ;;
esac
echo "  host: $(uname -sm) → $PLAT/$LIB"

echo "==> release build host cdylib"
cargo build --release --lib -p iroh-ffi

# cargo metadata serializes paths with platform-native separators; Git Bash
# expects forward slashes. Normalize.
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"].replace("\\", "/"))')

echo "==> stage to $RES/$PLAT/$LIB"
mkdir -p "$RES/$PLAT"
cp "$TARGET_DIR/release/$LIB" "$RES/$PLAT/$LIB"

# Move (not delete) the dev-mode flat lib aside; restore on exit. Otherwise
# `cargo make test-kotlin` afterwards would have to re-run bindgen-kotlin to
# put it back, and CI's tree-clean check would fire.
STASH=$(mktemp -d)
for f in libiroh_ffi.so libiroh_ffi.dylib iroh_ffi.dll; do
  [ -f "$RES/$f" ] && mv "$RES/$f" "$STASH/$f"
done
trap 'for f in libiroh_ffi.so libiroh_ffi.dylib iroh_ffi.dll; do [ -f "$STASH/$f" ] && mv "$STASH/$f" "$RES/$f"; done; rmdir "$STASH" 2>/dev/null || true' EXIT

echo "==> gradle test (must resolve $LIB from $PLAT/, not flat fallback)"
( cd kotlin && ./gradlew test --no-daemon --console=plain --quiet )
echo "verify-kotlin-consumer: OK"
