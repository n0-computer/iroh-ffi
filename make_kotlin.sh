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

cd kotlin

./gradlew generateNativeBindings

# copy cdylib to outdir
mkdir -p lib/src/main/resources/
cp ../target/debug/$LIB_NAME.$LIB_EXTENSION lib/src/main/resources/
