set -eu

# $CLASSPATH must include `jna` and `kotlinx-coroutines`

LIB_EXTENSION=""
LIB_NAME="libiroh_ffi"

case "$TEST_OS" in
    "mac")
        LIB_EXTENSION="dylib"
        ;;
    "linux")
        LIB_EXTENSION="so"
        ;;
    "windows")
        LIB_EXTENSION="lib"
        LIB_NAME="iroh_ffi"
        ;;
    *)
        echo "Unknown OS specified in TEST_OS"
        exit 1
        ;;
esac

echo "building library"
cargo build --lib

# UniFfi bindgen
echo "generating binding"
cargo run --bin uniffi-bindgen generate --language kotlin --out-dir kotlin/lib/src/main/kotlin/ --config uniffi.toml --library target/debug/$LIB_NAME.$LIB_EXTENSION

# copy cdylib to outdir
mkdir -p kotlin/lib/src/main/resources/
cp target/debug/$LIB_NAME.$LIB_EXTENSION kotlin/lib/src/main/resources/
