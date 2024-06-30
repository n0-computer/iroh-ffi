set -eu

# TODO: convert to rust

# Env
UDL_NAME="iroh"

# Compile the rust

# Needed on macos for ring
CC_aarch64_linux_android=aarch64-linux-android29-clang
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=aarch64-linux-android29-clang

echo "Building x86_64-linux-android"
cargo build --lib --target x86_64-linux-android
echo "Building i686-linux-android"
cargo build --lib --target i686-linux-android
echo "Building armv7-linux-androideabi"
cargo build --lib --target armv7-linux-androideabi
echo "Building aarch64-linux-android"
cargo build --lib --target aarch64-linux-android


# UniFfi bindgen
echo "generating binding"
cargo run --bin uniffi-bindgen generate "src/$UDL_NAME.udl" --language kotlin --out-dir ./kotlin --config uniffi.toml --lib-file  target/debug/libiroh.so
