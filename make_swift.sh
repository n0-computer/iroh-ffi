set -eu

# Builds the full 4-target Apple xcframework. Prefer `cargo make swift-xcframework`.

# Reproducible-build path normalization. Without this, every `.a` binary inside
# the xcframework embeds absolute paths from `file!()` macros (in deps, in
# iroh, and in std/core/alloc panic sites). Local builds carry
# `/Users/<you>/.cargo/...` + `/Users/<you>/.rustup/...`; CI carries
# `/Users/runner/...`. The same source on different hosts produces different
# byte streams (and hence a different `IrohLib.xcframework.zip` SHA-256). The
# four remaps below cover every absolute path rustc emits: cargo registry,
# cargo git deps, the source checkout, and the rustup-managed std sysroot.
# `scripts/release/zip_xcframework.sh` then packages the resulting (now
# host-independent) bytes into the deterministic zip whose checksum is baked
# into `Package.swift` by `cargo make prepare-release` and re-asserted by CI.
CARGO_PFX="${CARGO_HOME:-$HOME/.cargo}"
RUSTUP_PFX="${RUSTUP_HOME:-$HOME/.rustup}"
REPO_PFX="$(pwd)"
export RUSTFLAGS="${RUSTFLAGS:-} \
  --remap-path-prefix=${CARGO_PFX}/registry=/cargo/registry \
  --remap-path-prefix=${CARGO_PFX}/git=/cargo/git \
  --remap-path-prefix=${RUSTUP_PFX}=/rustup \
  --remap-path-prefix=${REPO_PFX}=/build \
  --crate-type=staticlib,cdylib"
# --remap-path-prefix is Rust-only. Several deps (notably `ring`) compile bundled
# C sources via build.rs + the `cc` crate, and those object files also embed
# absolute source paths. `-ffile-prefix-map` is clang/gcc's analogue. The `cc`
# crate forwards CFLAGS to every invocation.
export CFLAGS="${CFLAGS:-} \
  -ffile-prefix-map=${CARGO_PFX}/registry=/cargo/registry \
  -ffile-prefix-map=${CARGO_PFX}/git=/cargo/git \
  -ffile-prefix-map=${REPO_PFX}=/build"

# Apple deployment-target floors. The new iroh-rs deps call
# `nw_path_is_ultra_constrained` (iOS 17 / macOS 14); rustc's default
# `*-apple-ios` floor (10) and the unset macOS floor produce undefined-symbol
# link errors. Keep these in sync with Package.swift `platforms:`.
export IPHONEOS_DEPLOYMENT_TARGET="17.5"
export MACOSX_DEPLOYMENT_TARGET="14.5"

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

# The single root Package.swift consumes `Iroh.xcframework` at the repo root
# directly (IROH_LOCAL_XCFRAMEWORK mode), so no artifacts copy is needed.
