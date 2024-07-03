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
rm -rf ./kotlin/n0
cargo run --bin uniffi-bindgen generate --language kotlin --out-dir ./kotlin --config uniffi.toml --library target/debug/$LIB_NAME.$LIB_EXTENSION

# copy cdylib to outdir
cp ./target/debug/$LIB_NAME.$LIB_EXTENSION ./kotlin/


# Build jar file
echo "building jar"
rm -f ./kotlin/iroh_ffi.jar
kotlinc -Werror -d ./kotlin/iroh_ffi.jar ./kotlin/iroh_ffi/*.kt -classpath $CLASSPATH

# Execute Tests
echo "executing tests"
kotlinc -Werror -J-ea -classpath $CLASSPATH:./kotlin/iroh_ffi.jar:./kotlin -script ./kotlin/*.kts
