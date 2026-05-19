set -eu

# Build the Swift DocC archive and lay it out for static hosting under
# https://n0-computer.github.io/iroh-ffi/swift/. Prefer `cargo make docs-swift`.
#
# Needs the locally built xcframework (the Package.swift presence check picks
# it up); `cargo make docs-swift` depends on `swift-xcframework` so it is there.
#
# `swift test`-style SwiftPM cannot form the `Iroh` module on this setup, so
# docs are built through xcodebuild (the working toolchain), arch-forced to
# arm64 to match the arm64-only macOS xcframework slice. `docc
# transform-for-static-hosting` rewrites asset/base paths for the Pages
# subpath; only the deep documentation/ entry gets the correct baseUrl, so we
# replace the SPA-shell root index.html with a redirect into it.

SCHEME="IrohLib"
BASE_PATH="iroh-ffi/swift"
DDATA=".xcode-ddata-docs"
OUT="site/swift"

rm -rf "$DDATA" "$OUT"
mkdir -p site

arch -arm64 xcodebuild docbuild \
  -scheme "$SCHEME" \
  -destination 'platform=macOS' \
  -derivedDataPath "$DDATA"

ARCHIVE=$(find "$DDATA/Build/Products" -name '*.doccarchive' | head -1)
if [ -z "$ARCHIVE" ]; then
  echo "ERROR: no .doccarchive produced by xcodebuild docbuild" >&2
  exit 1
fi

DOCC=$(xcrun --find docc)
"$DOCC" process-archive transform-for-static-hosting "$ARCHIVE" \
  --hosting-base-path "$BASE_PATH" \
  --output-path "$OUT"

# DocC's root index.html is an SPA shell with baseUrl="/" (its assets 404 on a
# Pages subpath). The only correct entry is the deep documentation route, so
# redirect the root there.
cat > "$OUT/index.html" <<'HTML'
<!doctype html>
<meta charset="utf-8">
<title>iroh-ffi — Swift</title>
<meta http-equiv="refresh" content="0; url=documentation/irohlib/">
<link rel="canonical" href="documentation/irohlib/">
<a href="documentation/irohlib/">iroh-ffi Swift documentation</a>
HTML

rm -rf "$DDATA"
echo "swift docs -> $OUT (entry: documentation/irohlib/)"
