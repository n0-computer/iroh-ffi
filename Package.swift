// swift-tools-version:5.8
import PackageDescription

let package = Package(
    name: "IrohLib",
    platforms: [
        .iOS(.v15)
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
            path: "IrohLib/Sources/IrohLib"),
        .binaryTarget(
            name: "Iroh",
            url: "https://github.com/n0-computer/iroh-ffi/releases/download/v0.0.6/IrohLib.xcframework.zip",
            checksum: "4e612297d935332562ed8038ab6a66bde32dd644daf5f4d4f64e24f3bdf961e8")
    ]
)
