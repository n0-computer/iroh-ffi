#!/bin/bash
set -eu

# Deterministic zip of Iroh.xcframework — produces byte-identical output
# regardless of host or wall clock. Used by:
#   - `cargo make prepare-release` (local) so it can compute a checksum that
#     will match what CI uploads, and bake it into Package.swift in the same
#     PR (CI never rewrites main, per the Phase 6 plan).
#   - `.github/workflows/release.yml` `build-and-publish-swift` so the
#     uploaded asset matches the just-committed Package.swift checksum.
#
# Determinism recipe:
#   - normalize all mtimes to 1980-01-01 (the zip-format epoch);
#   - sort the file list with LC_ALL=C (locale-stable byte order);
#   - `zip -X` strips extra fields (UID/GID, hi-res mtime, etc).

if [ ! -d Iroh.xcframework ]; then
  echo "ERROR: Iroh.xcframework/ not present. Run cargo make swift-xcframework first." >&2
  exit 1
fi

rm -f IrohLib.xcframework.zip

# Zip stores mtimes with 2-second precision starting from 1980-01-01.
# Touch every entry (dirs too, on platforms that honour it) to that epoch.
find Iroh.xcframework -exec touch -t 198001010000 {} +

find Iroh.xcframework -print | LC_ALL=C sort | zip -X -q -y -@ IrohLib.xcframework.zip

CHECKSUM=$(shasum -a 256 IrohLib.xcframework.zip | awk '{print $1}')
SIZE=$(stat -f%z IrohLib.xcframework.zip 2>/dev/null || stat -c%s IrohLib.xcframework.zip)
echo "IrohLib.xcframework.zip  ($SIZE bytes)"
echo "sha256: $CHECKSUM"
