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

# Resolve target dir (honours CARGO_TARGET_DIR / .cargo config target-dir).
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')
LIB_PATH="$TARGET_DIR/debug/$LIB_NAME.$LIB_EXTENSION"

# UniFfi bindgen. The `Endpoint.close` -> `shutdown` rename for Kotlin is
# configured in `uniffi.toml` ([bindings.kotlin.rename]); no post-processing.
echo "generating binding"
cargo run --bin uniffi-bindgen generate --language kotlin --out-dir kotlin/lib/src/main/kotlin/ --config uniffi.toml --library "$LIB_PATH"

# copy cdylib to outdir
mkdir -p kotlin/lib/src/main/resources/
cp "$LIB_PATH" kotlin/lib/src/main/resources/
