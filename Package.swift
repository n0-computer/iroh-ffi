// swift-tools-version:5.8
import PackageDescription
import Foundation

// Single source of truth for the Swift package. The `Iroh` xcframework binary
// is resolved one of two ways, chosen at manifest-evaluation time:
//
//   * a locally built xcframework, when this is a source checkout that has
//     actually been built (`cargo make swift-xcframework` / `test-swift`).
//     This is what local dev, CI, and source consumers (e.g. an app pointing
//     at a local clone) get — so the binding always matches the source.
//   * otherwise (git-URL / Swift Package Index consumers): the pinned,
//     prebuilt xcframework zip attached to a GitHub release.
//
// Presence is keyed on the macOS slice *binary*, which is gitignored — the
// committed tree only carries an xcframework skeleton (Info.plist/Headers/
// Modules), so a fresh consumer checkout correctly falls through to the
// release zip. Set IROH_FORCE_REMOTE_XCFRAMEWORK to force the release zip
// even in a built checkout.
//
// The two release literals below are the only things `cargo make
// prepare-release` rewrites (per Phase 6 plan, CI never writes to main).
// Local prepare-release builds a deterministic xcframework zip, shasums it,
// and bakes both values into this manifest in the release commit.
let releaseTag = "v1.0.0-rc.1"
let releaseChecksum = "d0757677d9ff9184d1050b54625d5faaf7678ed70a69d33317c98d23426faab1"

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let localBuiltBinary = packageDir
    .appendingPathComponent("Iroh.xcframework/macos-arm64/Iroh.framework/Iroh")
let forceRemote = ProcessInfo.processInfo.environment["IROH_FORCE_REMOTE_XCFRAMEWORK"] != nil
let useLocalXcframework = !forceRemote
    && FileManager.default.fileExists(atPath: localBuiltBinary.path)

let irohBinary: Target = useLocalXcframework
    ? .binaryTarget(
        name: "Iroh",
        path: "Iroh.xcframework")
    : .binaryTarget(
        name: "Iroh",
        url: "https://github.com/n0-computer/iroh-ffi/releases/download/\(releaseTag)/IrohLib.xcframework.zip",
        checksum: releaseChecksum)

let package = Package(
    name: "IrohLib",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "IrohLib",
            targets: ["IrohLib", "Iroh"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "IrohLib",
            dependencies: [
                .byName(name: "Iroh")
            ],
            path: "IrohLib/Sources/IrohLib",
            linkerSettings: [
              .linkedFramework("SystemConfiguration"),
              // iroh's netdev uses Network.framework for interface enumeration
              // (the nw_* / nw_path_monitor_* symbols) on Apple platforms.
              .linkedFramework("Network"),
              // iroh's netwatch queries WiFi interfaces via CoreWLAN on macOS.
              .linkedFramework("CoreWLAN", .when(platforms: [.macOS]))
            ]),
        irohBinary,
        .testTarget(
            name: "IrohLibTests",
            dependencies: ["IrohLib"],
            path: "IrohLib/Tests/IrohLibTests"),
    ]
)
