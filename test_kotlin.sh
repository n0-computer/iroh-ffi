set -eu

# $CLASSPATH must include `jna`

echo "building library"
cargo build --lib

# UniFfi bindgen
echo "generating binding"
rm -rf ./kotlin/n0
cargo run --bin uniffi-bindgen generate "src/iroh.udl" --language kotlin --out-dir ./kotlin --config uniffi.toml

# copy cdylib to outdir
cp ./target/debug/libiroh.dylib ./kotlin/libuniffi_iroh.dylib

# Build jar file
echo "building jar"
rm -f ./kotlin/iroh.jar
kotlinc -Werror -d ./kotlin/iroh.jar ./kotlin/iroh/*.kt -classpath $CLASSPATH

# Execute Tests
echo "executing tests"
kotlinc -Werror -J-ea -classpath $CLASSPATH:./kotlin/iroh.jar:./kotlin -script ./kotlin/*.kts
