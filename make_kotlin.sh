set -eu

# Env
UDL_NAME="iroh"

# UniFfi bindgen
cargo run --bin uniffi-bindgen generate "src/$UDL_NAME.udl" --language kotlin  --out-dir ./kotlin

cargo build --target aarch64-linux-android --release