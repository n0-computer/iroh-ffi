#!/usr/bin/env bash
# SPIKE: two SEPARATE Swift modules over two separately-built native libraries sharing one
# copy of iroh.
#
#   IrohLib   (module) + iroh_ffiFFI      (C module) -> libiroh_ffi.dylib
#   IrohPing  (module) + iroh_ffi_pingFFI (C module) -> libiroh_ffi_ping.dylib
#
# uniffi documents that "you must compile all generated .swift files together in a single
# module since the generated code expects that it can access external types without
# importing them". That turns out NOT to be binding here: everything the plugin references
# from core (Endpoint, EndpointAddr, ProtocolHandler, FfiConverterType*_lower/_lift) is
# `public` in core's generated Swift, so one injected `import IrohLib` is enough.
#
# macOS + dylibs only. iOS needs dynamic frameworks — the current static xcframework model
# cannot work, see spikes/FINDINGS.md finding 15.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
TARGET_DIR="$REPO/target/debug"
STAGE="${STAGE:-$REPO/target/swift-spike}"
STD_DIR="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/lib"

echo "==> [1/4] static build (bindgen metadata only — see finding 9)"
cargo build -q -p iroh-ffi-ping

echo "==> [2/4] generating Swift bindings with the STOCK CLI"
GEN="$STAGE/.gen"
rm -rf "$STAGE"; mkdir -p "$GEN/core" "$GEN/ping"
cargo run -q --bin uniffi-bindgen -- generate --language swift \
  --library "$TARGET_DIR/libiroh_ffi.dylib"      --crate iroh_ffi      --out-dir "$GEN/core"
cargo run -q --bin uniffi-bindgen -- generate --language swift \
  --library "$TARGET_DIR/libiroh_ffi_ping.dylib" --crate iroh_ffi_ping --out-dir "$GEN/ping"

echo "==> [3/4] laying out the SwiftPM package"
mkdir -p "$STAGE"/Sources/{iroh_ffiFFI,iroh_ffi_pingFFI,IrohLib,IrohPing,SpikeMain}
cp "$GEN/core/iroh_ffiFFI.h"            "$STAGE/Sources/iroh_ffiFFI/"
cp "$GEN/core/iroh_ffiFFI.modulemap"    "$STAGE/Sources/iroh_ffiFFI/module.modulemap"
cp "$GEN/ping/iroh_ffi_pingFFI.h"       "$STAGE/Sources/iroh_ffi_pingFFI/"
cp "$GEN/ping/iroh_ffi_pingFFI.modulemap" "$STAGE/Sources/iroh_ffi_pingFFI/module.modulemap"
cp "$GEN/core/iroh_ffi.swift"           "$STAGE/Sources/IrohLib/"
# The one post-processing step the split needs.
{ printf 'import IrohLib\n'; cat "$GEN/ping/iroh_ffi_ping.swift"; } \
  > "$STAGE/Sources/IrohPing/iroh_ffi_ping.swift"
cp "$REPO/spikes/swift_main.swift" "$STAGE/Sources/SpikeMain/main.swift"

sed -e "s|@NATIVE_DIR@|$TARGET_DIR|g" -e "s|@STD_DIR@|$STD_DIR|g" \
  "$REPO/spikes/swift_package.swift.in" > "$STAGE/Package.swift"

echo "==> [4/4] dynamic build (the shipped artifacts) + swift build"
RUSTFLAGS="-C prefer-dynamic" cargo build -q -p iroh-ffi-ping
cd "$STAGE" && swift build

echo
echo "run: IROH_FORCE_STAGING_RELAYS=1 $STAGE/.build/debug/SpikeMain"
