#!/usr/bin/env bash
# JVM Maven artifact consumer smoke. Builds the host's release cdylib via an
# explicit cargo --target, stages it under its JNA-platform subdir (the path
# JNA discovers from classpath at runtime, same as release.yml's stage step),
# hides the dev-mode flat .so so the per-platform path must resolve, then
# runs the gradle test suite.
#
# Cross-platform — runs on linux/macOS/windows (uses Git Bash on win32).
# Lives outside Makefile.toml so cargo-make doesn't try to run the script
# through cmd.exe on Windows.

set -eu

RES=kotlin/lib/src/main/resources

# Hardware arch — immune to Rosetta on macOS (where uname can lie and report
# x86_64 even on an arm64 host whose JVM is the arm64 native build, leaving
# JNA looking at darwin-aarch64/ while we'd've staged to darwin-x86-64/).
case "$(uname -s)" in
  Darwin)
    case "$(sysctl -n hw.machine 2>/dev/null)" in
      arm64)  PLAT=darwin-aarch64 ; CARGO_TARGET=aarch64-apple-darwin ;;
      x86_64) PLAT=darwin-x86-64  ; CARGO_TARGET=x86_64-apple-darwin  ;;
      *) echo "ERROR: unsupported Darwin hw.machine=$(sysctl -n hw.machine)" >&2; exit 1 ;;
    esac
    LIB=libiroh_ffi.dylib
    ;;
  Linux)
    case "$(uname -m)" in
      x86_64)  PLAT=linux-x86-64  ; CARGO_TARGET=x86_64-unknown-linux-gnu  ;;
      aarch64) PLAT=linux-aarch64 ; CARGO_TARGET=aarch64-unknown-linux-gnu ;;
      *) echo "ERROR: unsupported Linux uname -m=$(uname -m)" >&2; exit 1 ;;
    esac
    LIB=libiroh_ffi.so
    ;;
  MINGW*|MSYS*)
    PLAT=win32-x86-64 ; LIB=iroh_ffi.dll ; CARGO_TARGET=x86_64-pc-windows-msvc ;;
  *)
    echo "ERROR: unsupported host $(uname -sm) — add a case branch" >&2 ; exit 1 ;;
esac
echo "  host: $(uname -s)/$(uname -m) → $PLAT/$LIB (cargo --target $CARGO_TARGET)"

echo "==> release build host cdylib for $CARGO_TARGET"
cargo build --release --lib -p iroh-ffi --target "$CARGO_TARGET"

# cargo metadata serializes paths with platform-native separators; Git Bash
# expects forward slashes. Normalize.
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"].replace("\\", "/"))')

echo "==> stage to $RES/$PLAT/$LIB"
mkdir -p "$RES/$PLAT"
cp "$TARGET_DIR/$CARGO_TARGET/release/$LIB" "$RES/$PLAT/$LIB"

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
