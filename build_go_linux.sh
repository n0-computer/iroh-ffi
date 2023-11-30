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

uniffi-bindgen-go $UDL_PATH --out-dir $GO_DIR

# TODO why does this needs to exist twice? once in the path and the other in
# the "deps" directory?
# move needed files over
mkdir ${INCLUDE_PATH}
cp "target/${DIR_NAME}/libiroh.so" "${INCLUDE_PATH}/libiroh.so"
mkdir "${INCLUDE_PATH}/deps"
cp "${INCLUDE_PATH}/libiroh.so" "${INCLUDE_PATH}/deps/libiroh.so"

# to run you need to let the linker know where the linked library files are:
# LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:./iroh/ffi" \
# CGO_LDFLAGS="-liroh -L ./iroh/ffi" \
# go <actual go command to build or run>
