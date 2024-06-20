set -eu

# $CLASSPATH must include `jna`

LIB_EXTENSION=""
LIB_NAME="libiroh"

case "$TEST_OS" in
    "mac")
        LIB_EXTENSION="dylib"
        ;;
    "linux")
        LIB_EXTENSION="so"
        ;;
    "windows")
        LIB_EXTENSION="lib"
        LIB_NAME="iroh"
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
cargo run --bin uniffi-bindgen generate "src/iroh.udl" --language kotlin --out-dir ./kotlin --config uniffi.toml

# copy cdylib to outdir
cp ./target/debug/$LIB_NAME.$LIB_EXTENSION ./kotlin/libuniffi_iroh.$LIB_EXTENSION

# Build jar file
echo "building jar"
rm -f ./kotlin/iroh.jar
kotlinc -Werror -d ./kotlin/iroh.jar ./kotlin/iroh/*.kt -classpath $CLASSPATH

# Execute Tests
echo "executing tests"
kotlinc -Werror -J-ea -classpath $CLASSPATH:./kotlin/iroh.jar:./kotlin -script ./kotlin/*.kts
