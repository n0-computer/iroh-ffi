#!/usr/bin/env bash
# Build the host iroh_ffi cdylib + run uniffi-bindgen for Kotlin + stage the
# host cdylib into kotlin/lib/src/main/resources/ for dev-mode JNA lookup.
#
# Extracted from Makefile.toml so the script body isn't run through cmd.exe
# on Windows (where cargo-make's `script_runner = "@shell"` resolves to
# cmd.exe and chokes on bash case/esac).

set -eu

# Env passed in from cargo-make: LIB_NAME=libiroh_ffi, UNIFFI_CONFIG=uniffi.toml.
: "${LIB_NAME:?LIB_NAME must be set}"
: "${UNIFFI_CONFIG:?UNIFFI_CONFIG must be set}"

case "$(uname -s)" in
  Darwin)        EXT=dylib ;;
  Linux)         EXT=so    ;;
  MINGW*|MSYS*)  EXT=dll   ;;
  *)             EXT=dll   ;;
esac

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"].replace("\\", "/"))')

# Windows dylib has no `lib` prefix.
case "$(uname -s)" in
  MINGW*|MSYS*) LIB="$TARGET_DIR/debug/iroh_ffi.${EXT}" ;;
  *)            LIB="$TARGET_DIR/debug/${LIB_NAME}.${EXT}" ;;
esac

cargo run --bin uniffi-bindgen generate --language kotlin \
  --out-dir kotlin/lib/src/main/kotlin/ --config "$UNIFFI_CONFIG" --library "$LIB"

mkdir -p kotlin/lib/src/main/resources/
cp "$LIB" kotlin/lib/src/main/resources/
