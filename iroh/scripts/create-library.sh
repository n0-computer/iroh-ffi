#!/usr/bin/env bash

# This should be invoked from inside xcode, not manually
if [ "$#" -ne 2 ]
then
    echo "Usage (note: only call inside xcode!):"
    echo "Args: $*"
    echo "path/to/build-scripts/xc-universal-binary.sh <CARGO_TOML_PATH>"
    exit 1
fi

# what to pass to cargo build -p, e.g. glean_ffi
FFI_TARGET=$1
# path to app services root
CARGO_TOML_PATH=$2
# buildvariant from our xcconfigs
#BUILDVARIANT=$3

RELFLAG=
if [[ "$CONFIGURATION" == "Release" ]]; then
#    rust_config="release"
#    env PATH="${build_path}" cargo +`cat ../rust-toolchain` build --release "${build_args[@]}"
    RELFLAG=--release
elif [[ "$CONFIGURATION" == "Debug" ]]; then
    echo "building for debug"
else
    echo "error: Unexpected build configuration: $CONFIGURATION"
fi
#if [[ "$BUILDVARIANT" != "debug" ]]; then
#    RELFLAG=--release
#fi

set -euvx

if [[ -n "${SDK_DIR:-}" ]]; then
  # Assume we're in Xcode, which means we're probably cross-compiling.
  # In this case, we need to add an extra library search path for build scripts and proc-macros,
  # which run on the host instead of the target.
  # (macOS Big Sur does not have linkable libraries in /usr/lib/.)
  export LIBRARY_PATH="${SDK_DIR}/usr/lib:${LIBRARY_PATH:-}"
fi

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
  IS_SIMULATOR=1
fi

for arch in $ARCHS; do
  case "$arch" in
    x86_64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        echo "Building for x86_64, but not a simulator build. What's going on?" >&2
        exit 2
      fi

      # Intel iOS simulator
      export CFLAGS_x86_64_apple_ios="-target x86_64-apple-ios"
      $HOME/.cargo/bin/cargo rustc -p $FFI_TARGET --lib --crate-type staticlib $RELFLAG --target x86_64-apple-ios --manifest-path $CARGO_TOML_PATH
      ;;

    arm64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        # Hardware iOS targets
        $HOME/.cargo/bin/cargo rustc -p $FFI_TARGET --lib --crate-type staticlib $RELFLAG --target aarch64-apple-ios --manifest-path $CARGO_TOML_PATH
      else
        # M1 iOS simulator -- currently in Nightly only and requires to build `libstd`
        $HOME/.cargo/bin/cargo rustc -p $FFI_TARGET --lib --crate-type staticlib $RELFLAG --target aarch64-apple-ios-sim --manifest-path $CARGO_TOML_PATH
      fi
  esac
done



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
#	env PATH="${build_path}" cargo +`cat ../rust-toolchain` build --release "${build_args[@]}"
#elif [[ "$CONFIGURATION" == "Debug" ]]; then
#	rust_config="debug"
#	env PATH="${build_path}" cargo +`cat ../rust-toolchain` build "${build_args[@]}"
#else
#    echo "error: Unexpected build configuration: $CONFIGURATION"
#fi
##
## Copy the built library to the derived files directory
#cp -v "./iroh-core-rust/iroh-ffi/target/${rust_target}/${rust_config}/libiroh.a" ${DERIVED_FILES_DIR}
