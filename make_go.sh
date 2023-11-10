set -eu

cargo build --release
rm -rf ./go/iroh/*
uniffi-bindgen-go ./src/iroh.udl --out-dir ./go
