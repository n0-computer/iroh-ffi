set -eu

# Builds the full 4-target Apple xcframework via `xcodebuild
# -create-xcframework -library`. Prefer `cargo make swift-xcframework`.
#
# Apple-blessed shape: each slice ships a flat `lib<name>.a` + `Headers/`
# directory containing the uniffi-generated FFI header, an `Export.h`
# umbrella, and a `module.modulemap` declaring the Swift-visible module
# name (`Iroh`). xcodebuild infers per-slice metadata (platform, arch,
# simulator vs device) from the .a's Mach-O headers and generates the
# outer xcframework Info.plist.
#
# Replaces the historical "checked-in framework skeleton + cp binaries
# into it" pattern, which forced per-Xcode-major hand-fixes of the
# bundle layout (see iroh-ffi#247: Xcode 27 rejected the shallow
# Info.plist layout iOS-style bundles use). With -library there is no
# .framework directory at all — just lib.a + headers — so that whole
# class of "bundle layout doesn't match the platform Apple expects"
# bug disappears.

# Reproducible-build path normalization. Without this, every `.a` binary
# embeds absolute paths from `file!()` macros (in deps, in iroh, and in
# std/core/alloc panic sites). The four remaps below cover every absolute
# path rustc emits: cargo registry, cargo git deps, the source checkout,
# and the rustup-managed std sysroot.
CARGO_PFX="${CARGO_HOME:-$HOME/.cargo}"
RUSTUP_PFX="${RUSTUP_HOME:-$HOME/.rustup}"
REPO_PFX="$(pwd)"
export RUSTFLAGS="${RUSTFLAGS:-} \
  --remap-path-prefix=${CARGO_PFX}/registry=/cargo/registry \
  --remap-path-prefix=${CARGO_PFX}/git=/cargo/git \
  --remap-path-prefix=${RUSTUP_PFX}=/rustup \
  --remap-path-prefix=${REPO_PFX}=/build"
# --remap-path-prefix is Rust-only. Several deps (notably `ring`) compile
# bundled C sources via build.rs + the `cc` crate; -ffile-prefix-map is
# clang/gcc's analogue.
export CFLAGS="${CFLAGS:-} \
  -ffile-prefix-map=${CARGO_PFX}/registry=/cargo/registry \
  -ffile-prefix-map=${CARGO_PFX}/git=/cargo/git \
  -ffile-prefix-map=${REPO_PFX}=/build"

# Apple deployment-target floors. iroh's netdev calls
# `nw_path_is_ultra_constrained` (iOS 17 / macOS 14); rustc's default
# `*-apple-ios` floor and the unset macOS floor produce undefined-symbol
# link errors at xcframework-consumption time on older SDKs.
export IPHONEOS_DEPLOYMENT_TARGET="17.5"
export MACOSX_DEPLOYMENT_TARGET="14.5"

UDL_NAME="iroh_ffi"
FRAMEWORK_NAME="Iroh"
SWIFT_INTERFACE="IrohLib"
INCLUDE_DIR="include/apple"

# Resolve the cargo target dir (honours CARGO_TARGET_DIR / .cargo config).
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')

# Default lib for the bindgen-metadata step (uniffi-bindgen reads symbols
# from a debug dylib to discover the FFI surface).
cargo build --lib

echo "Building aarch64-apple-ios"
cargo build --release --target aarch64-apple-ios
echo "Building aarch64-apple-ios-sim"
cargo build --release --target aarch64-apple-ios-sim
echo "Building x86_64-apple-ios"
cargo build --release --target x86_64-apple-ios
echo "Building aarch64-apple-darwin"
cargo build --release --target aarch64-apple-darwin
echo "Building aarch64-apple-ios-macabi"
cargo build --release --target aarch64-apple-ios-macabi

# Wipe outputs so we don't blend stale slices into the new xcframework.
rm -rf "$FRAMEWORK_NAME.xcframework"
rm -rf "$INCLUDE_DIR"
mkdir -p "$INCLUDE_DIR"

# UniFfi bindgen: produces ${UDL_NAME}FFI.h (C header for the FFI surface),
# ${UDL_NAME}.swift (the Swift binding code), and ${UDL_NAME}FFI.modulemap
# (a module declaration we ignore — we ship our own module.modulemap below
# that names the module `Iroh` to match what the Swift consumer imports).
cargo run --bin uniffi-bindgen generate --language swift --out-dir ./$INCLUDE_DIR --library "$TARGET_DIR/debug/lib${UDL_NAME}.dylib" --config uniffi.toml

# Stage a single headers directory shared across every slice — same .h +
# module.modulemap, so xcodebuild copies the same Headers/ into each slice.
# Export.h is a one-line umbrella so the modulemap can `umbrella header
# "Export.h"` without uniffi-generated names leaking into the module
# surface.
HEADERS_STAGE="$TARGET_DIR/apple-xcf-headers"
rm -rf "$HEADERS_STAGE"
mkdir -p "$HEADERS_STAGE"
cp "$INCLUDE_DIR/${UDL_NAME}FFI.h" "$HEADERS_STAGE/${UDL_NAME}FFI.h"
cat > "$HEADERS_STAGE/Export.h" <<EOF
#include "${UDL_NAME}FFI.h"
EOF
cat > "$HEADERS_STAGE/module.modulemap" <<EOF
module $FRAMEWORK_NAME {
    umbrella header "Export.h"
    export *
    module * { export * }
}
EOF

# Fat lib for the iOS simulator slice. xcframework can carry one .a per
# slice, so the arm64-sim and x86_64-sim variants need to be merged here.
# Name it lib${UDL_NAME}.a (not universal.a) so xcodebuild uses the same
# filename in every slice — consumers and the layout check both rely on it.
SIM_FAT="$TARGET_DIR/apple-sim-fat/lib${UDL_NAME}.a"
mkdir -p "$(dirname "$SIM_FAT")"
rm -f "$SIM_FAT"
lipo -create \
    "$TARGET_DIR/aarch64-apple-ios-sim/release/lib${UDL_NAME}.a" \
    "$TARGET_DIR/x86_64-apple-ios/release/lib${UDL_NAME}.a" \
    -output "$SIM_FAT"

# Assemble the xcframework. xcodebuild reads each .a's Mach-O headers to
# determine platform + arch + (simulator vs device), and emits the outer
# Info.plist + per-slice directories with the flat `lib<name>.a +
# Headers/` layout. No .framework bundles anywhere — Apple regenerates
# the AvailableLibraries metadata against the current Xcode SDK, so
# future Xcode majors don't need a hand-fix of the layout.
xcodebuild -create-xcframework \
    -library "$TARGET_DIR/aarch64-apple-ios/release/lib${UDL_NAME}.a" \
    -headers "$HEADERS_STAGE" \
    -library "$SIM_FAT" \
    -headers "$HEADERS_STAGE" \
    -library "$TARGET_DIR/aarch64-apple-darwin/release/lib${UDL_NAME}.a" \
    -headers "$HEADERS_STAGE" \
    -library "$TARGET_DIR/aarch64-apple-ios-macabi/release/lib${UDL_NAME}.a" \
    -headers "$HEADERS_STAGE" \
    -output "$FRAMEWORK_NAME.xcframework"

# Swift interface for the IrohLib SwiftPM target. uniffi emits references
# to the C module under its own name (${UDL_NAME}FFI); rewrite to the
# module name the consumer imports (`Iroh`).
sed "s/${UDL_NAME}FFI/$FRAMEWORK_NAME/g" "$INCLUDE_DIR/$UDL_NAME.swift" > "$INCLUDE_DIR/$SWIFT_INTERFACE.swift"
rm -f "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"
cp "$INCLUDE_DIR/$SWIFT_INTERFACE.swift" \
    "$SWIFT_INTERFACE/Sources/$SWIFT_INTERFACE/$SWIFT_INTERFACE.swift"
