#!/bin/bash

set -eu

MODE="--release"
DIR_NAME="release"

# the path to the new folder we are including
GO_DIR="./iroh-go"
IROH_GO_PATH="${GO_DIR}/iroh"
UDL_PATH="./src/iroh.udl"
IROH_GO_FILE="${IROH_GO_PATH}/iroh.go"

rm -rf $IROH_GO_PATH

# build iroh-ffi
cargo build $MODE 

# build go bindings
uniffi-bindgen-go $UDL_PATH --out-dir $GO_DIR

# move static library to the expected place
mv "target/${DIR_NAME}/libiroh.a" "${IROH_GO_PATH}/libiroh.a"

# Detect the operating system using uname
OS=$(uname -s)
SED='sed -i'
if [[ "$OS" == "Darwin" ]]; then
  SED='sed -i .temp'
fi
$SED 's|\/\/ #include <iroh.h>|\/\*\n#cgo windows LDFLAGS: -L${SRCDIR} -liroh\n#cgo linux LDFLAGS: -L${SRCDIR} -liroh -Wl,-unresolved-symbols=ignore-all\n#cgo darwin LDFLAGS: -L${SRCDIR} -liroh -Wl,-undefined,dynamic_lookup\n#include ".\/iroh.h"\n\*\/|' "$IROH_GO_FILE"
if [[ "$OS" == "Darwin" ]]; then
  rm $IROH_GO_FILE.temp
fi

echo "Build completed"
