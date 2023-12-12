#!/bin/bash

set -eu

MODE="--release"
DIR_NAME="release"

# the path to the new folder we are including
GO_DIR="./iroh-go"
INCLUDE_PATH="${GO_DIR}/iroh/ffi"
IROH_GO_PATH="${GO_DIR}/iroh/*"
UDL_PATH="./src/iroh.udl"
IROH_GO_FILE="${GO_DIR}/iroh/iroh.go"

rm -rf $IROH_GO_PATH

# build iroh-ffi and save the assets to ./go/iroh/include
cargo build $MODE 

uniffi-bindgen-go $UDL_PATH --out-dir $GO_DIR

# move needed files over
mkdir -p ${INCLUDE_PATH}
mkdir -p "${INCLUDE_PATH}/deps"

# Detect the operating system using uname
OS=$(uname -s)

if [[ "$OS" == "Darwin" ]]; then
  # macOS
  cp "target/${DIR_NAME}/libiroh.dylib" "${INCLUDE_PATH}/libiroh.dylib"
  sed -i '' "s/\/\/ #include <iroh.h>/\/\*\n#cgo CFLAGS: -I.\/ffi\n#cgo LDFLAGS: -liroh -L.\/ffi\n#include <iroh.h>\n\*\//" $IROH_GO_FILE
elif [[ "$OS" == "Linux" ]]; then
  # Linux
  cp "target/${DIR_NAME}/libiroh.so" "${INCLUDE_PATH}/libiroh.so"
  sed -i "s/\/\/ #include <iroh.h>/\/\*\n#cgo CFLAGS: -I.\/ffi\n#cgo LDFLAGS: -liroh -L.\/ffi\n#include <iroh.h>\n\*\//" $IROH_GO_FILE
else
  echo "Unsupported operating system: $OS"
  exit 1
fi

echo "Build completed for $OS"
