#!/usr/bin/env bash
# SPIKE: iOS. Builds core + plugin as DYNAMIC frameworks and runs a probe in the Simulator.
#
# Finding 15 showed the current static xcframework cannot host plugins: a plugin staticlib
# re-embeds all of iroh (688 MB) and collides on 83 `_ring_core_*` symbols. This is the
# dynamic-framework replacement.
#
#   Iroh.framework/Iroh              <- core, the only copy of iroh
#   IrohPing.framework/IrohPing      <- plugin, no iroh inside
#   RustStd.framework/RustStd        <- libstd, needed because everything is -C prefer-dynamic
#
# Frameworks (not loose dylibs) because that is what `xcodebuild -create-xcframework
# -framework` and SwiftPM `binaryTarget` consume.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
TRIPLE="${TRIPLE:-aarch64-apple-ios-sim}"
STAGE="${STAGE:-$REPO/target/ios-spike}"
export IPHONEOS_DEPLOYMENT_TARGET=17.5

echo "==> building for $TRIPLE"
RUSTFLAGS="-C prefer-dynamic" cargo build -q --target "$TRIPLE" -p ios-probe
D="$REPO/target/$TRIPLE/debug"
SR="$(rustc --print sysroot)"
STD_SRC=$(ls "$SR/lib/rustlib/$TRIPLE/lib/"libstd-*.dylib | head -1)

rm -rf "$STAGE"; mkdir -p "$STAGE/Frameworks"

# framework <fw-name> <source-dylib>
mk_framework() {
  local name="$1" src="$2" fw="$STAGE/Frameworks/$1.framework"
  mkdir -p "$fw"
  cp "$src" "$fw/$name"
  cat > "$fw/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>$name</string>
  <key>CFBundleIdentifier</key><string>computer.iroh.$name</string>
  <key>CFBundleName</key><string>$name</string>
  <key>CFBundlePackageType</key><string>FMWK</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>MinimumOSVersion</key><string>17.5</string>
</dict></plist>
PLIST
  install_name_tool -id "@rpath/$name.framework/$name" "$fw/$name"
}

echo "==> assembling frameworks"
mk_framework Iroh     "$D/libiroh_ffi.dylib"
mk_framework IrohPing "$D/libiroh_ffi_ping.dylib"
mk_framework RustStd  "$STD_SRC"

STD_BASE=$(basename "$STD_SRC")
# Rewrite every reference to the framework-relative form.
retarget() {
  local f="$1"
  for pair in "libiroh_ffi.dylib:Iroh.framework/Iroh" \
              "libiroh_ffi_ping.dylib:IrohPing.framework/IrohPing" \
              "$STD_BASE:RustStd.framework/RustStd"; do
    local dep="${pair%%:*}" new="${pair##*:}"
    local old
    old=$(otool -L "$f" | awk -v d="$dep" '$1 ~ d {print $1; exit}') || true
    if [ -n "${old:-}" ] && [ "$old" != "@rpath/$new" ]; then
      install_name_tool -change "$old" "@rpath/$new" "$f"
    fi
  done
}
retarget "$STAGE/Frameworks/Iroh.framework/Iroh"
retarget "$STAGE/Frameworks/IrohPing.framework/IrohPing"

cp "$D/ios-probe" "$STAGE/ios-probe"
retarget "$STAGE/ios-probe"
install_name_tool -add_rpath "@executable_path/Frameworks" "$STAGE/ios-probe" 2>/dev/null || true
codesign -f -s - "$STAGE"/Frameworks/*.framework "$STAGE/ios-probe" 2>/dev/null || true

echo "==> building an xcframework from the dynamic frameworks"
rm -rf "$STAGE/Iroh.xcframework"
xcodebuild -create-xcframework \
  -framework "$STAGE/Frameworks/Iroh.framework" \
  -output "$STAGE/Iroh.xcframework" >/dev/null

echo
echo "  framework sizes:"
for fw in Iroh IrohPing RustStd; do
  printf "    %-10s %12s bytes\n" "$fw" "$(stat -f%z "$STAGE/Frameworks/$fw.framework/$fw")"
done
echo "  probe deps:"; otool -L "$STAGE/ios-probe" | grep '@rpath' | sed 's/^/    /'
echo
echo "run: IROH_FORCE_STAGING_RELAYS=1 xcrun simctl spawn booted $STAGE/ios-probe"
