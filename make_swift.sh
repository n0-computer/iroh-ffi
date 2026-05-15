set -eu

# Builds the full 4-target Apple xcframework. Prefer `cargo make swift-xcframework`.

# Env
UDL_NAME="iroh_ffi"
FRAMEWORK_NAME="Iroh"
SWIFT_INTERFACE="IrohLib"
INCLUDE_DIR="include/apple"

# Resolve the cargo target dir (honours CARGO_TARGET_DIR / .cargo config).
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')

# Build default lib (for the bindgen step)
cargo build --lib

# Compile the rust
echo "Building aarch64-apple-ios"
cargo build --release --target aarch64-apple-ios
echo "Building aarch64-apple-ios-sim"
cargo build --release --target aarch64-apple-ios-sim
echo "Building x86_64-apple-ios"
cargo build --release --target x86_64-apple-ios
echo "Building aarch64-apple-darwin"
cargo build --release --target aarch64-apple-darwin

# Remove old files if they exist
IOS_ARM64_FRAMEWORK="$FRAMEWORK_NAME.xcframework/ios-arm64/$FRAMEWORK_NAME.framework"
IOS_SIM_FRAMEWORK="$FRAMEWORK_NAME.xcframework/ios-arm64_x86_64-simulator/$FRAMEWORK_NAME.framework"
MACOS_ARM64_FRAMEWORK="$FRAMEWORK_NAME.xcframework/macos-arm64/$FRAMEWORK_NAME.framework"

rm -f "$IOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"
rm -f "$IOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"
rm -f "$IOS_SIM_FRAMEWORK/$FRAMEWORK_NAME"
rm -f "$IOS_SIM_FRAMEWORK/Headers/${UDL_NAME}FFI.h"
rm -f "$MACOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"
rm -f "$MACOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"

rm -f "$TARGET_DIR/universal.a"
rm -f $INCLUDE_DIR/*

# Make dirs if it doesn't exist
mkdir -p $INCLUDE_DIR

# UniFfi bindgen
cargo run --bin uniffi-bindgen generate --language swift --out-dir ./$INCLUDE_DIR --library "$TARGET_DIR/debug/lib${UDL_NAME}.dylib" --config uniffi.toml

# Make fat lib for sims
lipo -create \
    "$TARGET_DIR/aarch64-apple-ios-sim/release/lib${UDL_NAME}.a" \
    "$TARGET_DIR/x86_64-apple-ios/release/lib${UDL_NAME}.a" \
    -output "$TARGET_DIR/universal.a"

# Move binaries
cp "$TARGET_DIR/aarch64-apple-ios/release/lib${UDL_NAME}.a" \
    "$IOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"
cp "$TARGET_DIR/universal.a" \
    "$IOS_SIM_FRAMEWORK/$FRAMEWORK_NAME"
cp "$TARGET_DIR/aarch64-apple-darwin/release/lib${UDL_NAME}.a" \
    "$MACOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"

# Move headers
cp "$INCLUDE_DIR/${UDL_NAME}FFI.h" \
    "$IOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"
cp "$INCLUDE_DIR/${UDL_NAME}FFI.h" \
    "$IOS_SIM_FRAMEWORK/Headers/${UDL_NAME}FFI.h"
cp "$INCLUDE_DIR/${UDL_NAME}FFI.h" \
    "$MACOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"

# Move swift interface
sed "s/${UDL_NAME}FFI/$FRAMEWORK_NAME/g" "$INCLUDE_DIR/$UDL_NAME.swift" > "$INCLUDE_DIR/$SWIFT_INTERFACE.swift"

rm -f "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"
cp "$INCLUDE_DIR/$SWIFT_INTERFACE.swift" \
    "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"

rm -rf "$SWIFT_INTERFACE/artifacts/Iroh.xcframework"
cp -R "$FRAMEWORK_NAME.xcframework" "$SWIFT_INTERFACE/artifacts/"
