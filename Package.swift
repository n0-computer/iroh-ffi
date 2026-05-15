// swift-tools-version:5.8
import PackageDescription
import Foundation

// Single source of truth for the Swift package. The `Iroh` xcframework binary
// is resolved one of two ways, chosen at manifest-evaluation time:
//
//   * default (consumers / Swift Package Index): the pinned, prebuilt
//     xcframework zip attached to a GitHub release.
//   * IROH_LOCAL_XCFRAMEWORK set (local dev / CI): the freshly built
//     `Iroh.xcframework` at the repo root (produced by `cargo make
//     swift-xcframework`).
//
// The two release literals below are the only things the release workflow
// rewrites — there is no second manifest to keep in sync.
let releaseTag = "v0.20.0"
let releaseChecksum = "8123c2d43690c423e9bc8993c935b2fe009731f3b65b95754358570077037858"

let irohBinary: Target = ProcessInfo.processInfo.environment["IROH_LOCAL_XCFRAMEWORK"] != nil
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
