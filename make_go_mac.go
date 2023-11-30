set -eu

MODE=""
DIR_NAME="debug"
if [ "$#" -eq 1 ]; then
  if [[ $1 == "release" ]]; then
    MODE="--release"
    DIR_NAME="release"
  elif [[ $1 != "debug" ]]; then
    echo "Unknown mode '$1'. Options are 'release' and 'debug'. Defaults to 'debug'"
    exit
  fi
fi

# the path to the new folder we are including
GO_DIR="./iroh-go"
INCLUDE_PATH="${GO_DIR}/iroh/ffi"
IROH_GO_PATH="${GO_DIR}/iroh/*"
UDL_PATH="./src/iroh.udl"
IROH_GO_FILE="${GO_DIR}/iroh/iroh.go"

rm -rf $IROH_GO_PATH

# build iroh-ffi and save the assets to ./go/iroh/include
cargo build $MODE 

# TODO why does this needs to exist twice? once in the path and the other in
# the "deps" directory?
# move needed files over
cp "target/${DIR_NAME}/libiroh.dylib" "${INCLUDE_PATH}/libiroh.dylib"
mkdir "${INCLUDE_PATH}/deps"
cp "${INCLUDE_PATH}"/libiroh.dylib" "${INCLUDE_PATH}/deps/libiroh.dylib"

uniffi-bindgen-go $UDL_PATH --out-dir $GO_DIR

sed -i '' "s/\/\/ #include <iroh.h>/\/\*\n#cgo CFLAGS: -I.\/ffi\n#cgo LDFLAGS: -liroh -L.\/ffi\n#include <iroh.h>\n\*\//" $IROH_GO_FILE
