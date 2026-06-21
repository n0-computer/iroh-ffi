set -eu

# Build ONLY the macOS-arm64 slice of the Iroh framework — enough for
# `xcodebuild docbuild` (DocC) and nothing more. The full 4-target
# xcframework (`make_swift.sh`) builds three extra iOS slices that DocC
# never consumes; skipping them cuts the docs build from ~13min of release
# Rust builds to ~3min. Used only by `cargo make docs-swift`.

UDL_NAME="iroh_ffi"
FRAMEWORK_NAME="Iroh"
SWIFT_INTERFACE="IrohLib"
INCLUDE_DIR="include/apple"

export MACOSX_DEPLOYMENT_TARGET="14.5"

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')

# Default lib (for the bindgen metadata step) + the macOS release slice.
cargo build --lib
echo "Building aarch64-apple-darwin"
cargo build --release --target aarch64-apple-darwin

MACOS_ARM64_FRAMEWORK="$FRAMEWORK_NAME.xcframework/macos-arm64/$FRAMEWORK_NAME.framework"
MACOS_ARM64_FRAMEWORK_BIN="$MACOS_ARM64_FRAMEWORK/Versions/A/$FRAMEWORK_NAME"
rm -f "$MACOS_ARM64_FRAMEWORK_BIN"
rm -f "$MACOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"
rm -f $INCLUDE_DIR/*
mkdir -p $INCLUDE_DIR

# UniFfi bindgen (Swift interface + FFI header)
cargo run --bin uniffi-bindgen generate --language swift --out-dir ./$INCLUDE_DIR --library "$TARGET_DIR/debug/lib${UDL_NAME}.dylib" --config uniffi.toml

cp "$TARGET_DIR/aarch64-apple-darwin/release/lib${UDL_NAME}.a" \
  "$MACOS_ARM64_FRAMEWORK_BIN"
cp "$INCLUDE_DIR/${UDL_NAME}FFI.h" \
  "$MACOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"

# Swift interface consumed by the IrohLib target.
sed "s/${UDL_NAME}FFI/$FRAMEWORK_NAME/g" "$INCLUDE_DIR/$UDL_NAME.swift" >"$INCLUDE_DIR/$SWIFT_INTERFACE.swift"
rm -f "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"
cp "$INCLUDE_DIR/$SWIFT_INTERFACE.swift" \
  "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"

echo "macos-arm64 slice ready -> $MACOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"
