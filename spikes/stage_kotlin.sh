#!/usr/bin/env bash
# SPIKE: stage two *separate* Kotlin/JVM modules that share ONE native copy of iroh.
#
#   kotlin/lib   -> computer.iroh        + libiroh_ffi.dylib + libstd
#   kotlin/ping  -> computer.iroh.ping   + libiroh_ffi_ping.dylib (no iroh inside)
#
# Two builds are needed, and the order matters:
#   1. static build  -> the plugin dylib contains BOTH crates' UNIFFI_META, so the STOCK
#                       `uniffi-bindgen` CLI can resolve external types. No custom bindgen.
#   2. dynamic build -> the artifacts we actually ship, one copy of iroh.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
TARGET_DIR="$REPO/target/debug"
LIB_RES="$REPO/kotlin/lib/src/main/resources"
PING_RES="$REPO/kotlin/ping/src/main/resources"
PING_SRC="$REPO/kotlin/ping/src/main/kotlin"

case "$(uname -s)" in Darwin) EXT=dylib ;; Linux) EXT=so ;; *) echo "macOS/Linux only" >&2; exit 1 ;; esac

echo "==> [1/4] static build (for bindgen metadata only)"
cargo build -q -p iroh-ffi-ping

echo "==> [2/4] generating bindings with the STOCK CLI"
# No --config: it would override the config for EVERY crate, so core would inherit the
# plugin's package_name and the plugin would import its own package instead of core's.
mkdir -p "$PING_SRC"
cargo run -q --bin uniffi-bindgen -- generate --language kotlin --no-format \
  --library "$TARGET_DIR/libiroh_ffi.$EXT"      --crate iroh_ffi \
  --out-dir "$REPO/kotlin/lib/src/main/kotlin"
cargo run -q --bin uniffi-bindgen -- generate --language kotlin --no-format \
  --library "$TARGET_DIR/libiroh_ffi_ping.$EXT" --crate iroh_ffi_ping \
  --out-dir "$PING_SRC"

echo "==> [3/4] dynamic build (the shipped artifacts)"
RUSTFLAGS="-C prefer-dynamic" cargo build -q -p iroh-ffi-ping

echo "==> [4/4] staging natives"
mkdir -p "$LIB_RES" "$PING_RES"
cp "$TARGET_DIR/libiroh_ffi.$EXT"      "$LIB_RES/"
cp "$TARGET_DIR/libiroh_ffi_ping.$EXT" "$PING_RES/"
STD_LIB=$(find "$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/lib" \
            -name "libstd-*.$EXT" | head -1)
cp "$STD_LIB" "$LIB_RES/"

if [ "$EXT" = "dylib" ]; then
  # Core resolves libstd next to itself. The plugin needs NO rpath to core: dyld satisfies
  # `@rpath/libiroh_ffi.dylib` from the already-loaded image, and JNA loads core first
  # because computer.iroh.ping's bindings touch computer.iroh types. This is what makes the
  # split survive JNA extracting natives to mangled temp filenames.
  install_name_tool -id "@rpath/libiroh_ffi.dylib" "$LIB_RES/libiroh_ffi.dylib"
  install_name_tool -add_rpath "@loader_path" "$LIB_RES/libiroh_ffi.dylib" 2>/dev/null || true
  OLD=$(otool -L "$PING_RES/libiroh_ffi_ping.dylib" | awk '/libiroh_ffi\.dylib/{print $1;exit}')
  install_name_tool -change "$OLD" "@rpath/libiroh_ffi.dylib" "$PING_RES/libiroh_ffi_ping.dylib"
  install_name_tool -id "@rpath/libiroh_ffi_ping.dylib" "$PING_RES/libiroh_ffi_ping.dylib"
  install_name_tool -add_rpath "@loader_path/../../../../lib/src/main/resources" \
    "$PING_RES/libiroh_ffi_ping.dylib" 2>/dev/null || true
  codesign -f -s - "$LIB_RES/libiroh_ffi.dylib" "$PING_RES/libiroh_ffi_ping.dylib" 2>/dev/null || true
fi

echo
echo "core:   $(ls -la "$LIB_RES/libiroh_ffi.$EXT"       | awk '{print $5}') bytes"
echo "plugin: $(ls -la "$PING_RES/libiroh_ffi_ping.$EXT" | awk '{print $5}') bytes"
