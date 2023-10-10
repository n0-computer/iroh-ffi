set -eu

cargo build
rm -rf ./go/iroh/*
uniffi-bindgen-go ./src/iroh.udl --out-dir ./go
