// swift-tools-version:5.9
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
// Presence is keyed on the macOS slice's static lib. The whole xcframework
// directory is gitignored (build artifact only — Apple regenerates it from
// the cargo-built .a files via `xcodebuild -create-xcframework -library`),
// so a fresh consumer checkout has nothing local to find and falls through
// to the release zip. Set IROH_FORCE_REMOTE_XCFRAMEWORK to force the release
// zip even in a built checkout.
//
// `releaseTag` is rewritten locally by `cargo make prepare-release <V>`.
// `releaseChecksum` is rewritten on PR CI by `release_swift.yml` (one bot
// commit on the release branch, marked `[skip swift-release]`), so the value
// here always matches the IrohLib.xcframework.zip attached to the GitHub
// release — never a cross-host determinism game.
let releaseTag = "v1.1.0"
let releaseChecksum = "ad46dadf09f9224157512992923562931ed60f252414230d50893a4d515c5776"

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let localBuiltBinary = packageDir
    .appendingPathComponent("Iroh.xcframework/macos-arm64/libiroh_ffi.a")
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
        .iOS("17.5"),
        .macOS("14.5"),
        .macCatalyst("17.5")
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
