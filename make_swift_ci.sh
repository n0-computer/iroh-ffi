set -eu

# TODO: convert to rust

# Env
UDL_NAME="iroh"
FRAMEWORK_NAME="Iroh"
SWIFT_INTERFACE="IrohLib"

rm -f include/swift/*

# Make dirs if it doesn't exist
mkdir -p include/swift

# UniFfi bindgen
cargo run --bin uniffi-bindgen generate "src/$UDL_NAME.udl" --language swift --out-dir ./include/swift

## Move lib into proper location
## Run swift test

# # Make fat lib for sims
# lipo -create \
#     "./target/aarch64-apple-ios-sim/release/lib${UDL_NAME}.a" \
#     "./target/x86_64-apple-ios/release/lib${UDL_NAME}.a" \
#     -output ./target/universal.a

# # Move binaries
# cp "./target/aarch64-apple-ios/release/lib${UDL_NAME}.a" \
#     "$IOS_ARM64_FRAMEWORK/$FRAMEWORK_NAME"
# cp ./target/universal.a \
#     "$IOS_SIM_FRAMEWORK/$FRAMEWORK_NAME"

# # Move headers
# cp "include/ios/${UDL_NAME}FFI.h" \
#     "$IOS_ARM64_FRAMEWORK/Headers/${UDL_NAME}FFI.h"
# cp "include/ios/${UDL_NAME}FFI.h" \
#     "$IOS_SIM_FRAMEWORK/Headers/${UDL_NAME}FFI.h"

# # Move swift interface
# sed "s/${UDL_NAME}FFI/$FRAMEWORK_NAME/g" "include/ios/$UDL_NAME.swift" > "include/ios/$SWIFT_INTERFACE.swift"

# rm -f "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"
# cp "include/ios/$SWIFT_INTERFACE.swift" \
#     "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"

# rm -rf "$SWIFT_INTERFACE/artifacts/*"
# cp -R "$FRAMEWORK_NAME.xcframework" "$SWIFT_INTERFACE/artifacts/"
