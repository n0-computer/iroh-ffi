set -eu

# TODO: convert to rust

# Env
UDL_NAME="iroh"
FRAMEWORK_NAME="Iroh"
SWIFT_INTERFACE="IrohLib"
INCLUDE_DIR="include/apple"

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

rm -f ./target/universal.a
rm -f $INCLUDE_DIR/*

# Make dirs if it doesn't exist
mkdir -p $INCLUDE_DIR

# UniFfi bindgen
cargo run --bin uniffi-bindgen generate "src/$UDL_NAME.udl" --language swift --out-dir ./$INCLUDE_DIR

# Make fat lib for sims
lipo -create \
    "./target/aarch64-apple-ios-sim/release/lib${UDL_NAME}.a" \
    "./target/x86_64-apple-ios/release/lib${UDL_NAME}.a" \
    -output ./target/universal.a

# Move binaries
cp "./target/aarch64-apple-ios/release/lib${UDL_NAME}.a" \
    "$IOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"
cp ./target/universal.a \
    "$IOS_SIM_FRAMEWORK/$FRAMEWORK_NAME"
cp "./target/aarch64-apple-darwin/release/lib${UDL_NAME}.a" \
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

rm -rf "$SWIFT_INTERFACE/artifacts/*"
cp -R "$FRAMEWORK_NAME.xcframework" "$SWIFT_INTERFACE/artifacts/"
