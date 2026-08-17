#!/usr/bin/env bash
# SPIKE: the COMPOSITION case — four Python packages, a diamond of four native libraries.
#
#   iroh/         libiroh_ffi.dylib + libstd    (the only copy of iroh)
#   iroh_blobs/   libiroh_ffi_blobs.dylib
#   iroh_gossip/  libiroh_ffi_gossip.dylib
#   iroh_docs/    libiroh_ffi_docs.dylib        -> depends on ALL THREE
#
# Two builds, as in stage_kotlin.sh: static for bindgen metadata, dynamic to ship.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
T="$REPO/target/debug"
STAGE="${STAGE:-$REPO/target/docs-spike}"

echo "==> [1/4] static build (bindgen metadata for all four namespaces)"
cargo build -q -p iroh-ffi-docs

echo "==> [2/4] generating bindings (stock CLI, one library, four namespaces)"
rm -rf "$STAGE"; mkdir -p "$STAGE"/{iroh,iroh_blobs,iroh_gossip,iroh_docs}
cargo run -q --bin uniffi-bindgen -- generate --language python \
  --library "$T/libiroh_ffi_docs.dylib" --crate iroh_ffi        --out-dir "$STAGE/iroh"
cargo run -q --bin uniffi-bindgen -- generate --language python \
  --library "$T/libiroh_ffi_docs.dylib" --crate iroh_ffi_blobs  --out-dir "$STAGE/iroh_blobs"
cargo run -q --bin uniffi-bindgen -- generate --language python \
  --library "$T/libiroh_ffi_docs.dylib" --crate iroh_ffi_gossip --out-dir "$STAGE/iroh_gossip"
cargo run -q --bin uniffi-bindgen -- generate --language python \
  --library "$T/libiroh_ffi_docs.dylib" --crate iroh_ffi_docs   --out-dir "$STAGE/iroh_docs"

echo "==> [3/4] dynamic build (one copy of everything)"
RUSTFLAGS="-C prefer-dynamic" cargo build -q -p iroh-ffi-docs

echo "==> [4/4] staging natives"
cp "$T/libiroh_ffi.dylib"        "$STAGE/iroh/"
cp "$T/libiroh_ffi_blobs.dylib"  "$STAGE/iroh_blobs/"
cp "$T/libiroh_ffi_gossip.dylib" "$STAGE/iroh_gossip/"
cp "$T/libiroh_ffi_docs.dylib"   "$STAGE/iroh_docs/"
STD_LIB=$(find "$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/lib" \
            -name "libstd-*.dylib" | head -1)
cp "$STD_LIB" "$STAGE/iroh/"

# plain pairs — macOS ships bash 3.2, which has no associative arrays
for pair in libiroh_ffi:iroh libiroh_ffi_blobs:iroh_blobs \
            libiroh_ffi_gossip:iroh_gossip libiroh_ffi_docs:iroh_docs; do
  lib="${pair%%:*}"; dir="${pair##*:}"
  f="$STAGE/$dir/$lib.dylib"
  install_name_tool -id "@rpath/$lib.dylib" "$f"
  # Repoint every absolute build path at @rpath, and add an rpath per sibling package.
  for dep in libiroh_ffi libiroh_ffi_blobs libiroh_ffi_gossip; do
    OLD=$(otool -L "$f" | awk -v d="/$dep.dylib" '$1 ~ d {print $1; exit}')
    if [ -n "${OLD:-}" ] && [ "${OLD#@rpath}" = "$OLD" ]; then
      install_name_tool -change "$OLD" "@rpath/$dep.dylib" "$f"
    fi
  done
  install_name_tool -add_rpath "@loader_path" "$f" 2>/dev/null || true
  for sib in iroh iroh_blobs iroh_gossip; do
    install_name_tool -add_rpath "@loader_path/../$sib" "$f" 2>/dev/null || true
  done
  codesign -f -s - "$f" 2>/dev/null || true
done

for p in iroh iroh_blobs iroh_gossip iroh_docs; do
  ns=$(ls "$STAGE/$p" | grep '\.py$' | head -1 | sed 's/\.py$//')
  printf 'from .%s import *  # NOQA\n' "$ns" > "$STAGE/$p/__init__.py"
done
cp "$REPO/spikes/docs_round_trip.py" "$STAGE/"

echo
for p in iroh iroh_blobs iroh_gossip iroh_docs; do
  printf "  %-13s %s\n" "$p/" "$(ls "$STAGE/$p" | tr '\n' ' ')"
done
echo
echo "run: IROH_FORCE_STAGING_RELAYS=1 python3 $STAGE/docs_round_trip.py"
