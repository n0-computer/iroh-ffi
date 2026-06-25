#!/usr/bin/env bash
# Upload IrohLib.xcframework.zip to the draft GitHub release v<version>.
# Parses <version> from Package.swift's `releaseTag` literal. Creates the
# draft release if one doesn't exist; replaces the asset if it does.
#
# Invoked by .github/workflows/release_swift.yml. Locally:
#
#   gh auth login   # or `export GH_TOKEN=…`
#   bash scripts/release/swift_upload_draft.sh

set -eu

ZIP=IrohLib.xcframework.zip
[ -f "$ZIP" ] || { echo "ERROR: $ZIP not found" >&2; exit 1; }

V=$(grep -oE 'let releaseTag = "v[^"]+"' Package.swift | sed -E 's/.*"v([^"]+)"/\1/')
[ -n "$V" ] || { echo "ERROR: couldn't parse releaseTag in Package.swift" >&2; exit 1; }

if ! gh release view "v$V" >/dev/null 2>&1; then
  gh release create "v$V" \
    --draft \
    --title "v$V" \
    --notes "Draft release — promoted to published by release.yml on tag push."
fi
gh release upload "v$V" "$ZIP" --clobber
