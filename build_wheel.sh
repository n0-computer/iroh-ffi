#!/bin/sh

# Build a manylinux_2_28 (default) or musllinux_1_2 wheel
#
# docker run --rm -v $(pwd):/mnt -w /mnt quay.io/pypa/manylinux_2_28_x86_64 /mnt/build_wheel.sh
# docker run --rm -v $(pwd):/mnt -w /mnt quay.io/pypa/musllinux_1_2_x86_64 /mnt/build_wheel.sh musllinux

set -e

case "${1:-manylinux}" in
    manylinux) COMPAT_ARGS="--manylinux 2_28" ;;
    musllinux) COMPAT_ARGS="--compatibility musllinux_1_2" ;;
    *)         echo "usage: $0 [manylinux|musllinux]" >&2; exit 2 ;;
esac

# Install rust
# Note: this adds cargo to PATH, which is needed since uniffi-bindgen
# needs to be built from source on musllinux.
curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source $HOME/.cargo/env

# Build just for python 3.11
PATH=/opt/python/cp311-cp311/bin:$PATH
export PATH

# Install our build tools
pip install maturin uniffi-bindgen

# Build the wheel
maturin build --release $COMPAT_ARGS
