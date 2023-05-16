##!/bin/bash
#set -x
## create-iospw.sh
## Build the correct Rust target and place
## the resulting library in the build products
##
## The $PATH used by Xcode likely won't contain Cargo, fix that.
## In addition, the $PATH used by XCode has lots of Apple-specific
## developer tools that your Cargo isn't expecting to use, fix that.
## Note: This assumes a default `rustup` setup and default path.
#build_path="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
#
##export CFLAGS_x86_64_apple_darwin="-I /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/include"
##export CFLAGS_aarch64_apple_ios_sim="-I /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk/usr/include"
##export CFLAGS_aarch64_apple_ios="-I /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/usr/include"
#
##export LIBRARY_PATH="/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib"
##export LIBRARY_PATH="/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk/usr/include"
#
##echo $(gcc -Xlinker -v)
#
#if [[ -n "${DEVELOPER_SDK_DIR:-}" ]]; then
#  # Assume we're in Xcode, which means we're probably cross-compiling.
#  # In this case, we need to add an extra library search path for build scripts and proc-macros,
#  # which run on the host instead of the target.
#  # (macOS Big Sur does not have linkable libraries in /usr/lib/.)
#  export LIBRARY_PATH="${DEVELOPER_SDK_DIR}/MacOSX.sdk/usr/lib:${LIBRARY_PATH:-}"
#fi
#
##
## Figure out the correct Rust target from the ARCHS and PLATFORM.
## This script expects just one element in ARCHS.
#case "$ARCHS" in
#	"arm64")	rust_arch="aarch64" ;;
#	"x86_64")	rust_arch="x86_64" ;;
#	*)			echo "error: unsupported architecture: $ARCHS" ;;
#esac
#if [[ "$PLATFORM_NAME" == "macosx" ]]; then
#	rust_platform="apple-darwin"
#else
#	rust_platform="apple-ios"
#fi
#if [[ "$PLATFORM_NAME" == "iphonesimulator" ]]; then
#    if [[ "${rust_arch}" == "aarch64" ]]; then
#        rust_abi="-sim"
#    else
#        rust_abi=""
#    fi
#else
#	rust_abi=""
#fi
#rust_target="${rust_arch}-${rust_platform}${rust_abi}"
##
## Build library in debug or release
#build_args=(--manifest-path ./iroh-core-rust/iroh-ffi/Cargo.toml --target "${rust_target}")
#if [[ "$CONFIGURATION" == "Release" ]]; then
#	rust_config="release"
#	env PATH="${build_path}" RUSTFLAGS="-C lto=on -C embed-bitcode=yes" cargo +`cat ./rust-toolchain` build --release "${build_args[@]}"
#elif [[ "$CONFIGURATION" == "Debug" ]]; then
#	rust_config="debug"
#	env PATH="${build_path}" RUSTFLAGS="-C embed-bitcode=yes" cargo +`cat ./rust-toolchain` build "${build_args[@]}"
#else
#    echo "error: Unexpected build configuration: $CONFIGURATION"
#fi
##
## Copy the built library to the derived files directory
##cp -v "./iroh-core-rust/iroh-ffi/target/${rust_target}/${rust_config}/libiroh.a" ${DERIVED_FILES_DIR}



export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export PATH="$HOME/.cargo/bin:$PATH"

FILENAME="libiroh.a"
DIR="."

# Delete old build, if any.
#rm -f "${DIR}/${FILENAME}"

rustc `cat ./rust-toolchain` --version

# ensure all targets are installed
rustup target add aarch64-apple-ios x86_64-apple-ios --toolchain `cat ./rust-toolchain`

# --xcode-integ determines --release and --targets from Xcode's env vars.
# Depending your setup, specify the rustup toolchain explicitly.
RUSTFLAGS="-C lto=on -C embed-bitcode=yes" \
  cargo +`cat ./rust-toolchain` lipo --xcode-integ --manifest-path "$DIR/iroh-core-rust/iroh-ffi/Cargo.toml"

# cargo-lipo drops result in different folder, depending on the config.
if [[ $CONFIGURATION = "Debug" ]]; then
  SOURCE="$DIR/iroh-core-rust/target/universal/debug/${FILENAME}"
else
  SOURCE="$DIR/iroh-core-rust/target/universal/release/${FILENAME}"
fi

## Copy compiled library to DIR.
#if [ -e "${SOURCE}" ]; then
#  cp -a "${SOURCE}" $DIR
#fi
