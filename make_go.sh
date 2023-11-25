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
INCLUDE_PATH="${GO_DIR}/iroh/include"
IROH_GO_PATH="${GO_DIR}/iroh/*"
UDL_PATH="./src/iroh.udl"
IROH_GO_FILE="${GO_DIR}/iroh/iroh.go"

rm -rf $IROH_GO_PATH

# build iroh-ffi and save the assets to ./go/iroh/include
cargo build $MODE --target-dir $INCLUDE_PATH

uniffi-bindgen-go $UDL_PATH --out-dir $GO_DIR

sed -i '' "s/\/\/ #include <iroh.h>/\/\*\n#cgo CFLAGS: -I.\/include\/${DIR_NAME}\n#cgo LDFLAGS: -liroh -L.\/include\/${DIR_NAME}\n#include <iroh.h>\n\*\//" $IROH_GO_FILE
