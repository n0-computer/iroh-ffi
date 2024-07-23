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
            url: "https://github.com/n0-computer/iroh-ffi/releases/download/v0.21.0/IrohLib.xcframework.zip",
            checksum: "f8bfb2c9cdc9d602494e25e204ada6d44a7d6032de695a97893dca3c7fa3fac6")
    ]
)
