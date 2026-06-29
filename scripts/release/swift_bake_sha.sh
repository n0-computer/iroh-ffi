#!/usr/bin/env bash
# Bake the SHA-256 of IrohLib.xcframework.zip into Package.swift, commit the
# change to the current branch, and push. No-op if the SHA hasn't changed.
#
# Invoked by .github/workflows/release_swift.yml on push to release/v*. Can
# also be run locally for dry-runs as long as you've just built the zip:
#
#   cargo make swift-xcframework
#   bash scripts/release/zip_xcframework.sh
#   bash scripts/release/swift_bake_sha.sh
#
# Locally, `git push` will fail unless you're on a branch with an upstream;
# that's fine — the commit + diff is what you want to review.

set -eu

ZIP=IrohLib.xcframework.zip
[ -f "$ZIP" ] || { echo "ERROR: $ZIP not found (run scripts/release/zip_xcframework.sh first)" >&2; exit 1; }

CURRENT=$(grep -oE 'let releaseChecksum = "[a-f0-9]+"' Package.swift | sed -E 's/.*"([a-f0-9]+)"/\1/')
NEW=$(shasum -a 256 "$ZIP" | awk '{print $1}')

if [ "$CURRENT" = "$NEW" ]; then
  echo "checksum unchanged ($NEW) — nothing to commit"
  exit 0
fi

python3 scripts/release/bump_version.py --checksum "$NEW"

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add Package.swift
git commit -m "chore(release): bake Swift xcframework SHA $NEW [skip swift-release]"
git push
