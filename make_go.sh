set -eu

cargo build --release
cp taget/release/libiroh.dylib ./go/iroh/include/libiroh.dylib
uniffi-bindgen-go ./src/iroh.udl --out-dir ./go
