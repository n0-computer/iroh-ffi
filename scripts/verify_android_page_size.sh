#!/usr/bin/env bash
# Assert every Android .so under $1 uses 16 KB LOAD alignment (align 2**14).
# Google Play blocks apps targeting Android 15+ whose native libs are
# 4 KB-aligned (align 2**12). See #279.
#
# Reads `LLVM/binutils readelf -lW` (segment headers, wide format) and greps
# the LOAD lines for their alignment. Fails on the first .so that doesn't
# match, printing the offending header block for diagnosis.

set -eu

ROOT="${1:-./android-jniLibs}"
[ -d "$ROOT" ] || { echo "ERROR: $ROOT not found" >&2; exit 1; }

if command -v readelf >/dev/null 2>&1; then
  READELF=readelf
elif command -v llvm-readelf >/dev/null 2>&1; then
  READELF=llvm-readelf
else
  echo "ERROR: neither readelf nor llvm-readelf on PATH" >&2
  exit 1
fi

fail=0
for so in "$ROOT"/*/libiroh_ffi.so; do
  [ -f "$so" ] || continue
  # LOAD alignment field is the last column of each LOAD row. `2**14 (0x4000)`
  # is 16 KB; `2**12` is the offending 4 KB default.
  bad=$("$READELF" -lW "$so" | awk '/^  LOAD/ && $NF != "0x4000" { print }')
  if [ -n "$bad" ]; then
    echo "FAIL: $so has non-16KB LOAD alignment" >&2
    echo "$bad" >&2
    fail=1
  else
    echo "OK: $so (16 KB LOAD alignment)"
  fi
done

exit "$fail"
