#!/usr/bin/env bash
# SPIKE: two SEPARATE napi .node addons sharing ONE copy of iroh.
#
#   iroh/       js_iroh_core.node  + libjs_iroh.dylib + libiroh_ffi.dylib + libstd
#   iroh_ping/  js_iroh_ping.node
#
# The napi types (Endpoint, EndpointAddr) live in libjs_iroh.dylib, NOT in either addon.
# That is what makes `napi_unwrap` across addons sound: both see the same monomorphization.
# It also puts napi itself in exactly one place, so its module registry is process-global.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
TARGET_DIR="$REPO/target/debug"
STAGE="${STAGE:-$REPO/target/js-spike}"

echo "==> building (prefer-dynamic so std is shared too — see finding 2)"
RUSTFLAGS="-C prefer-dynamic" cargo build -q -p js-iroh-core -p js-iroh-ping

rm -rf "$STAGE"; mkdir -p "$STAGE/iroh" "$STAGE/iroh_ping"
cp "$TARGET_DIR/libjs_iroh_core.dylib" "$STAGE/iroh/js_iroh_core.node"
cp "$TARGET_DIR/libjs_iroh.dylib"      "$STAGE/iroh/"
cp "$TARGET_DIR/libiroh_ffi.dylib"     "$STAGE/iroh/"
cp "$TARGET_DIR/libjs_iroh_ping.dylib" "$STAGE/iroh_ping/js_iroh_ping.node"
STD_LIB=$(find "$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/lib" \
            -name "libstd-*.dylib" | head -1)
cp "$STD_LIB" "$STAGE/iroh/"

echo "==> fixing install names / rpaths"
for lib in libjs_iroh libiroh_ffi; do
  install_name_tool -id "@rpath/$lib.dylib" "$STAGE/iroh/$lib.dylib"
  install_name_tool -add_rpath "@loader_path" "$STAGE/iroh/$lib.dylib" 2>/dev/null || true
done
# Repoint absolute build paths at @rpath in every consumer.
for f in "$STAGE/iroh/js_iroh_core.node" "$STAGE/iroh_ping/js_iroh_ping.node" \
         "$STAGE/iroh/libjs_iroh.dylib"; do
  for dep in libjs_iroh libiroh_ffi; do
    OLD=$(otool -L "$f" | awk -v d="$dep.dylib" '$1 ~ d {print $1; exit}')
    [ -n "${OLD:-}" ] && [ "${OLD#@rpath}" = "$OLD" ] && \
      install_name_tool -change "$OLD" "@rpath/$dep.dylib" "$f"
  done
done
install_name_tool -add_rpath "@loader_path"          "$STAGE/iroh/js_iroh_core.node" 2>/dev/null || true
install_name_tool -add_rpath "@loader_path/../iroh"  "$STAGE/iroh_ping/js_iroh_ping.node" 2>/dev/null || true
codesign -f -s - "$STAGE"/iroh/*.dylib "$STAGE"/iroh/*.node "$STAGE"/iroh_ping/*.node 2>/dev/null || true

cp "$REPO/spikes/js_round_trip.mjs" "$STAGE/"
echo
ls -la "$STAGE/iroh" "$STAGE/iroh_ping" | awk '{printf "  %-34s %12s\n",$9,$5}'
echo
echo "run: IROH_FORCE_STAGING_RELAYS=1 node $STAGE/js_round_trip.mjs"
