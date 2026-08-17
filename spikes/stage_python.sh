#!/usr/bin/env bash
# SPIKE: stage two *separate* Python packages that share ONE native copy of iroh.
#
#   <stage>/iroh/       libiroh_ffi.dylib + libstd + iroh_ffi.py     (the "core wheel")
#   <stage>/iroh_ping/  libiroh_ffi_ping.dylib + iroh_ffi_ping.py    (the "plugin wheel")
#
# The plugin dylib carries NO copy of iroh; it resolves core through @rpath at load time.
set -euo pipefail

PROFILE="${1:-debug}"
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$REPO/target/$PROFILE"
STAGE="${STAGE:-$REPO/target/spike-stage}"

case "$(uname -s)" in
  Darwin) EXT=dylib ;;
  Linux)  EXT=so ;;
  *) echo "spike is macOS/Linux only" >&2; exit 1 ;;
esac

CORE_LIB="$TARGET_DIR/libiroh_ffi.$EXT"
PLUGIN_LIB="$TARGET_DIR/libiroh_ffi_ping.$EXT"
[ -f "$CORE_LIB" ]   || { echo "missing $CORE_LIB — run cargo build first" >&2; exit 1; }
[ -f "$PLUGIN_LIB" ] || { echo "missing $PLUGIN_LIB — run cargo build first" >&2; exit 1; }

rm -rf "$STAGE"
mkdir -p "$STAGE/iroh" "$STAGE/iroh_ping"

echo "==> generating core bindings (namespace iroh_ffi)"
cargo run --quiet --bin uniffi-bindgen -- generate \
  --language python --library "$CORE_LIB" --config "$REPO/uniffi.toml" \
  --out-dir "$STAGE/iroh"

echo "==> generating plugin bindings (namespace iroh_ffi_ping, external types -> iroh)"
# NOT the stock CLI: it reads UNIFFI_META from one library only, so it cannot resolve
# `iroh_ffi::endpoint`. spike-bindgen merges core's metadata in. See spikes/bindgen/.
cargo run --quiet -p spike-bindgen -- \
  "$PLUGIN_LIB" "$STAGE/iroh_ping" iroh_ffi_ping "$CORE_LIB"

echo "==> copying native libraries"
cp "$CORE_LIB"   "$STAGE/iroh/"
cp "$PLUGIN_LIB" "$STAGE/iroh_ping/"

# libstd must ship: -C prefer-dynamic semantics apply to any Rust `dylib`, and both
# artifacts declare @rpath/libstd-<hash>.dylib. The hash is tied to the exact rustc.
STD_LIB=$(find "$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/lib" \
            -name "libstd-*.$EXT" | head -1)
echo "==> shipping $(basename "$STD_LIB")"
cp "$STD_LIB" "$STAGE/iroh/"

if [ "$EXT" = "dylib" ]; then
  echo "==> fixing macOS install names / rpaths"
  # core: identify by @rpath, find libstd next to itself
  install_name_tool -id "@rpath/libiroh_ffi.dylib" "$STAGE/iroh/libiroh_ffi.dylib"
  install_name_tool -add_rpath "@loader_path" "$STAGE/iroh/libiroh_ffi.dylib" 2>/dev/null || true

  # plugin: repoint the absolute build path at @rpath, then look in ../iroh
  OLD_CORE_REF=$(otool -L "$STAGE/iroh_ping/libiroh_ffi_ping.dylib" \
                   | awk '/libiroh_ffi\.dylib/ {print $1; exit}')
  install_name_tool -change "$OLD_CORE_REF" "@rpath/libiroh_ffi.dylib" \
    "$STAGE/iroh_ping/libiroh_ffi_ping.dylib"
  install_name_tool -id "@rpath/libiroh_ffi_ping.dylib" "$STAGE/iroh_ping/libiroh_ffi_ping.dylib"
  install_name_tool -add_rpath "@loader_path/../iroh" "$STAGE/iroh_ping/libiroh_ffi_ping.dylib" 2>/dev/null || true
  install_name_tool -add_rpath "@loader_path" "$STAGE/iroh_ping/libiroh_ffi_ping.dylib" 2>/dev/null || true
  codesign -f -s - "$STAGE/iroh/libiroh_ffi.dylib" 2>/dev/null || true
  codesign -f -s - "$STAGE/iroh_ping/libiroh_ffi_ping.dylib" 2>/dev/null || true
else
  patchelf --set-rpath '$ORIGIN'                   "$STAGE/iroh/libiroh_ffi.so"
  patchelf --set-rpath '$ORIGIN:$ORIGIN/../iroh'   "$STAGE/iroh_ping/libiroh_ffi_ping.so"
fi

printf 'from .iroh_ffi import *  # NOQA\n'       > "$STAGE/iroh/__init__.py"
printf 'from .iroh_ffi_ping import *  # NOQA\n'  > "$STAGE/iroh_ping/__init__.py"

echo
echo "staged in $STAGE"
ls -la "$STAGE/iroh" "$STAGE/iroh_ping" | awk '{printf "  %-34s %10s\n", $9, $5}'
