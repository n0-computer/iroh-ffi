set -eu

cargo build
uniffi-bindgen-go ./src/iroh.udl --out-dir ./go
