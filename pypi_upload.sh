#!/bin/bash

# script to batch upload our wheels to pypi and test pypi
# expects the 5 platforms/architecture we support to exist in the same folder, which you pass in as an argument (darwin x86, darwin arm, win arm, linux aarch, linux x86)
# script is only confirmed to work on mac

# Function to print usage
usage() {
    echo "Usage: $0 <version> <path> --test"
    echo "  <version>    Version number in format x.x.x"
    echo "  <path>       Path to the directory where the wheels are located."
    echo "  --test       Optional flag to upload to test PyPI"
    exit 1
}

# Check if at least two arguments are provided
if [ "$#" -lt 2 ]; then
    usage
fi

# Extract positional arguments
version=$1
path=$2
test=false

# Check for the test flag
if [ "$#" -eq 3 ] && [ "$3" == "--test" ]; then
    test=true
elif [ "$#" -gt 3 ]; then
    usage
fi

# Validate version format (x.x.x)
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in the format x.x.x"
    exit 1
fi

# Check if path exists
if [ ! -e "$path" ]; then
    echo "Error: Path '$path' does not exist"
    exit 1
fi

# Upload using twine
if $test; then
    echo "Uploading to test PyPI..."
    twine upload --repository testpypi --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-macosx_10_12_x86_64.whl"
    twine upload --repository testpypi --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-macosx_11_0_arm64.whl"
    twine upload --repository testpypi --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-manylinux_2_17_aarch64.manylinux2014_aarch64.whl"
    twine upload --repository testpypi --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-manylinux_2_17_x86_64.manylinux2014_x86_64.whl"
    twine upload --repository testpypi --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-win_amd64.whl"
else
    echo "Uploading to PyPI..."
    twine upload --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-macosx_10_12_x86_64.whl"
    twine upload --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-macosx_11_0_arm64.whl"
    twine upload --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-manylinux_2_17_aarch64.manylinux2014_aarch64.whl"
    twine upload --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-manylinux_2_17_x86_64.manylinux2014_x86_64.whl"
    twine upload --config-file ~/.pypirc "${path}/iroh-${version}-py3-none-win_amd64.whl"
fi

echo "Upload complete."
